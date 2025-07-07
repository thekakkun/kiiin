use std::process::Command;

use axum::{
    Router,
    http::StatusCode,
    routing::{get, post},
};

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/check", get(check))
        .route("/text", post(handle_text));

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
