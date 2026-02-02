use axum::{Json, Router, extract::State, http::Method, http::header, routing::get};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
mod auth;
mod models;
use auth::AuthUser;

#[derive(Clone)]
struct AppState {
    db: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    println!("Connection to the database is successful");

    let state = AppState { db: pool };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/hello", get(hello_handler))
        .route("/api/users", get(get_users_handler))
        .route("/api/me", get(get_me_handler))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("🚀 Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

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

async fn get_users_handler(State(state): State<AppState>) -> Json<Vec<String>> {
    let _pool = state.db;
    Json(vec!["DB is available".to_string()])
}

async fn get_me_handler(AuthUser(claims): AuthUser) -> Json<String> {
    Json(format!("You are authenticated. Email: {}, ID: {}", claims.email, claims.sub))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*; // modelsモジュールを使う
    use ts_rs::TS;

    #[test]
    fn export_bindings() {
        // 出力先のディレクトリが存在しないとエラーになることがあるので、
        // 必要ならfs::create_dir_allなどで作る処理を入れてもいい。
        // ここでは単純にexportを実行する。
        
        // ユーザー定義型の書き出し
        User::export().expect("Failed to export User struct");
        Room::export().expect("Failed to export Room struct");
        CreateRoomRequest::export().expect("Failed to export CreateRoomRequest");
        
        println!("🎉 TypeScript bindings exported successfully!");
    }
}