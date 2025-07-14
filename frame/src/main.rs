use std::{env, fs::File, process::Command};

use axum::{
    Router,
    extract::Multipart,
    http::StatusCode,
    routing::{get, post},
};
use image::ImageFormat;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/check", get(check))
        .route("/text", post(handle_text))
        .route("/image", post(handle_image));

    // run it
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn check() -> &'static str {
    "Hello, World!"
}

async fn handle_text(body: String) -> StatusCode {
    let _ = Command::new("eips")
        .arg("-c")
        .stdout(std::process::Stdio::null())
        .status();
    let _ = Command::new("eips")
        .arg(body)
        .stdout(std::process::Stdio::null())
        .status();

    StatusCode::OK
}

async fn handle_image(mut multipart: Multipart) -> StatusCode {
    let dir = env::var("DIR").unwrap_or("/mnt/us/extensions/kiiin_frame".to_string());
    while let Some(field) = multipart.next_field().await.unwrap() {
        let format = match field.content_type() {
            Some("image/bmp") => ImageFormat::Bmp,
            Some("image/png") => ImageFormat::Png,
            _ => return StatusCode::UNSUPPORTED_MEDIA_TYPE,
        };

        let data = field.bytes().await.unwrap();
        let img = image::load_from_memory_with_format(&data, format).unwrap();

        let mut file = match File::create(format!("{dir}/image")) {
            Ok(f) => f,
            Err(e) => return StatusCode::INTERNAL_SERVER_ERROR,
        };
        if let Err(e) = img.write_to(&mut file, format) {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }

        let mut eips = Command::new("eips");
        let _ = match format {
            ImageFormat::Bmp => eips
                .args(["-b", "image"])
                .stdout(std::process::Stdio::null())
                .status(),
            ImageFormat::Png => eips
                .args(["-g", "image"])
                .stdout(std::process::Stdio::null())
                .status(),
            _ => unreachable!(),
        };
    }

    StatusCode::OK
}
