use std::{env, fs::File, process::Command};

use axum::{
    Router,
    extract::{Multipart, multipart::Field},
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
    "OK"
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
    let mut img_format = ImageFormat::Png;
    let mut refresh = false;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap();

        match name {
            "file" => img_format = save_image(field).await.unwrap(),
            "refresh" => refresh = field.text().await.unwrap() == "true",
            &_ => println!("Unknown field: {}", name),
        }
    }
    let mut eips_cmd = Command::new("eips");

    if refresh {
        eips_cmd.arg("-f");
    };

    match img_format {
        ImageFormat::Bmp => eips_cmd.args(["-b", "/mnt/us/image"]),
        ImageFormat::Png => eips_cmd.args(["-g", "/mnt/us/image"]),
        _ => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    match eips_cmd.output() {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn save_image(field: Field<'_>) -> Result<ImageFormat, StatusCode> {
    let dir = env::var("DIR").unwrap_or("/mnt/us/".to_string());
    let img_format = match field.content_type() {
        Some("image/bmp") => ImageFormat::Bmp,
        Some("image/png") => ImageFormat::Png,
        _ => return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE),
    };

    let data = match field.bytes().await {
        Ok(data) => data,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let img = match image::load_from_memory_with_format(&data, img_format) {
        Ok(img) => img,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let mut file = match File::create(format!("{dir}/image")) {
        Ok(f) => f,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match img.write_to(&mut file, img_format) {
        Ok(_) => Ok(img_format),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
