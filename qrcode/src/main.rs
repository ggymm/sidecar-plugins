use std::env;
use std::process;

use rqrr::PreparedImage;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <qrcode image path>", args[0]);
        process::exit(1);
    }

    let path = &args[1];

    // 读取图片
    let img = match image::open(path) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("can not open image: {} {}", path, e);
            process::exit(1);
        }
    };

    let img = img.to_luma8();
    let mut img = PreparedImage::prepare(img);

    let grids = img.detect_grids();
    if grids.is_empty() {
        eprintln!("can not find qrcode in image: {}", path);
        process::exit(1);
    }

    let (_, content) = match grids[0].decode() {
        Ok(decoded) => decoded,
        Err(e) => {
            eprintln!("can not decode qrcode in image: {} {}", path, e);
            process::exit(1);
        }
    };
    println!("{}", content);
}
