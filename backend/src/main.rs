use axum::{
    http::Method,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/hello", get(hello_handler))
        .layer(cors);

    // サーバーを起動
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("🚀 Server listening on {}", addr);

    // --- ▼ここからが修正箇所！▼ ---

    // 1. axum::Server::bind(...) の代わりに、tokio::net::TcpListener を使う
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // 2. .serve(...) の代わりに、axum::serve を使う
    axum::serve(listener, app).await.unwrap();

    // --- ▲ここまで！▲ ---
}

// (hello_handler と HelloResponse 構造体は変更なし)
#[derive(Serialize)]
struct HelloResponse {
    message: String,
}

async fn hello_handler() -> Json<HelloResponse> {
    let response = HelloResponse {
        message: "Hello from Rust & Axum! 🦀".to_string(),
    };
    Json(response)
}