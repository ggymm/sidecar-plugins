use axum::{
    body::Body,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use qrcode::QrCode;
use std::env;
use std::fs::File;
use std::io::{Cursor, SeekFrom};
use std::net::{TcpListener as StdTcpListener, UdpSocket};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::net::TcpListener;
use tokio_util::io::ReaderStream;
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

    // 获取 IP 地址
    let mut local_ip = "127.0.0.1".to_string();
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                local_ip = addr.ip().to_string();
            }
        }
    }

    // 获取随机端口号
    let local_tcp = StdTcpListener::bind("127.0.0.1:0")?;
    let local_port = local_tcp.local_addr()?.port();
    drop(local_tcp);

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
    let file_read = File::open(file_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let file_size = file_read
        .metadata()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();
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

    let len = end - start + 1;
    let mut file = tokio::fs::File::open(file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let stream = ReaderStream::new(file.take(len));

    if start != 0 || end != file_size - 1 {
        Ok((
            StatusCode::PARTIAL_CONTENT,
            [
                (header::ACCEPT_RANGES, "bytes"),
                (header::CONTENT_RANGE, &format!("bytes {}-{}/{}", start, end, file_size)),
                (header::CONTENT_LENGTH, &len.to_string()),
                (header::CONTENT_TYPE, &content_type),
            ],
            Body::from_stream(stream),
        )
            .into_response())
    } else {
        Ok((
            StatusCode::OK,
            [
                (header::ACCEPT_RANGES, "bytes"),
                (header::CONTENT_LENGTH, &len.to_string()),
                (header::CONTENT_TYPE, &content_type),
            ],
            Body::from_stream(stream),
        )
            .into_response())
    }
}

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;

    #[test]
    fn test_local_ip() {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.connect("8.8.8.8:80").unwrap();
        let local_addr = socket.local_addr().unwrap();
        println!("local_addr: {}", local_addr);
    }
}
