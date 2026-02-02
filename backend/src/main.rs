use crate::models::{CreateRoomRequest, Room};
use auth::AuthUser;
use axum::{
    extract::State,
    http::header,
    http::Method,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
mod auth;
mod models;

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
        .route("/api/room/create", post(create_room_handler))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from((
        [0, 0, 0, 0],
        std::env::var("BACKEND_PORT")
            .expect("BACKEND_PORT must be set")
            .parse::<u16>()
            .expect("Port is not integer"),
    ));
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

/// 認証確認
async fn get_me_handler(AuthUser(claims): AuthUser) -> Json<String> {
    Json(format!(
        "You are authenticated. Email: {}, ID: {}",
        claims.email.unwrap_or_else(|| "UNDEFINED".to_string()),
        claims.sub
    ))
}

/// ルーム作成
async fn create_room_handler(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(payload): Json<CreateRoomRequest>,
) -> Result<Json<Room>, (axum::http::StatusCode, String)> {
    let room = sqlx::query_as::<_, Room>(
        r#"
        INSERT INTO rooms (name, owner_id)
        VALUES ($1, (SELECT id FROM users WHERE firebase_uid = $2))
        RETURNING id, name, owner_id, created_at
        "#,
    )
    .bind(payload.name)
    .bind(claims.sub) // FirebaseのUIDをキーにowner_idを特定
    .fetch_one(&state.db)
    .await
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(room))
}

/// ユーザー情報を同期する（いなければ作成、いれば最新情報に更新）
pub async fn sync_user(
    db: &Pool<Postgres>,
    claims: &auth::Claims,
) -> Result<uuid::Uuid, sqlx::Error> {
    // query! ではなく query を使用。型チェックは実行時に行われる。
    let record = sqlx::query(
        r#"
        INSERT INTO users (firebase_uid, email, display_name, photo_url)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (firebase_uid) 
        DO UPDATE SET 
            email = EXCLUDED.email,
            display_name = EXCLUDED.display_name,
            photo_url = EXCLUDED.photo_url,
            updated_at = CURRENT_TIMESTAMP
        RETURNING id
        "#,
    )
    .bind(&claims.sub) // sub は String なのでそのまま
    .bind(&claims.email) // email は Option<String> なので、SQL側では NULL 許容になる
    .bind(&claims.name) // 同上
    .bind(&claims.picture) // 同上
    .fetch_one(db)
    .await?;

    // query_as を使わない場合は、手動で ID を取り出す必要がある
    use sqlx::Row;
    let id: uuid::Uuid = record.get("id");

    Ok(id)
}
