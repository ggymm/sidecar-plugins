use axum::{
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use qrcode::QrCode;
use std::env;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::net::{IpAddr, TcpListener as StdTcpListener};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

struct AppState {
    file_path: PathBuf,
    download_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]);
    if !file_path.exists() || !file_path.is_file() {
        eprintln!("Error: '{}' is not a valid file", args[1]);
        std::process::exit(1);
    }

    let mut local_ip = "127.0.0.1".to_string();
    for interface in if_addrs::get_if_addrs()? {
        if !interface.is_loopback() {
            match interface.ip() {
                IpAddr::V4(ipv4) => {
                    if !ipv4.is_loopback() && !ipv4.is_multicast() && !ipv4.is_link_local() {
                        local_ip = ipv4.to_string()
                    }
                }
                IpAddr::V6(_) => continue,
            }
        }
    }

    let local_bind = StdTcpListener::bind("127.0.0.1:0")?;
    let local_port = local_bind.local_addr()?.port();
    drop(local_bind);

    let file_id = Uuid::new_v4().to_string();
    let base_url = format!("http://{}:{}", local_ip, local_port);
    println!(
        r#"{{"pid":{},"file_id":"{}","base_url":"{}"}}"#,
        std::process::id(),
        file_id,
        base_url
    );

    let app = Router::new()
        .route("/qrcode", get(qrcode))
        .route(&format!("/{}", file_id), get(download))
        .with_state(Arc::new(AppState {
            file_path,
            download_url: format!("{}/{}", base_url, file_id),
        }));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", local_port)).await?;
    tokio::spawn(async {
        tokio::signal::ctrl_c().await.unwrap();
        std::process::exit(0);
    });

    axum::serve(listener, app).await?;
    Ok(())
}

async fn qrcode(axum::extract::State(state): axum::extract::State<Arc<AppState>>) -> Result<Response, StatusCode> {
    let code = QrCode::new(&state.download_url).unwrap();
    let image = code
        .render::<image::Luma<u8>>()
        .min_dimensions(600, 600)
        .max_dimensions(600, 600)
        .build();

    let mut png_data = Vec::new();
    image::DynamicImage::ImageLuma8(image)
        .write_to(&mut Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/png"),
            (header::CONTENT_LENGTH, &png_data.len().to_string()),
        ],
        png_data,
    )
        .into_response())
}

async fn download(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let file_path = &state.file_path;
    let file = File::open(file_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let file_size = file.metadata().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.len();
    let content_type = mime_guess::from_path(file_path).first_or_octet_stream().to_string();

    let (start, end) = if let Some(range_header) = headers.get(header::RANGE) {
        let range_str = range_header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
        if !range_str.starts_with("bytes=") {
            (0, file_size - 1)
        } else {
            let range_spec = &range_str[6..];
            if let Some(dash_pos) = range_spec.find('-') {
                let start_str = &range_spec[..dash_pos];
                let end_str = &range_spec[dash_pos + 1..];

                let start = if start_str.is_empty() {
                    if let Ok(suffix_len) = end_str.parse::<u64>() {
                        file_size.saturating_sub(suffix_len)
                    } else {
                        0
                    }
                } else {
                    start_str.parse::<u64>().unwrap_or(0).min(file_size - 1)
                };

                let end = if end_str.is_empty() {
                    file_size - 1
                } else {
                    end_str.parse::<u64>().unwrap_or(file_size - 1).min(file_size - 1)
                };

                (start, end.max(start))
            } else {
                (0, file_size - 1)
            }
        }
    } else {
        (0, file_size - 1)
    };

    // 读取文件内容
    let mut file = File::open(file_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.seek(SeekFrom::Start(start))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let length = end - start + 1;
    let mut buffer = vec![0u8; length as usize];
    file.read_exact(&mut buffer)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if start != 0 || end != file_size - 1 {
        Ok((
            StatusCode::PARTIAL_CONTENT,
            [
                (header::ACCEPT_RANGES, "bytes"),
                (header::CONTENT_RANGE, &format!("bytes {}-{}/{}", start, end, file_size)),
                (header::CONTENT_LENGTH, &length.to_string()),
                (header::CONTENT_TYPE, &content_type),
            ],
            buffer,
        )
            .into_response())
    } else {
        Ok((
            StatusCode::OK,
            [
                (header::ACCEPT_RANGES, "bytes"),
                (header::CONTENT_LENGTH, &length.to_string()),
                (header::CONTENT_TYPE, &content_type),
            ],
            buffer,
        )
            .into_response())
    }
}
