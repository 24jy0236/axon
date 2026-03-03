use axum::{
    extract::State,
    http::{header, Method},
    routing::{get, post},
    Json, Router,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use ts_rs::TS;

mod auth;
mod entities; // 作成したEntityモジュール

use auth::AuthUser;
use entities::{prelude::*, *}; // Entityを使うためのインポート

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection, // Pool<Postgres> ではなく SeaORM のコネクション
}

// リクエストDTO
#[derive(Deserialize, TS)]
#[ts(
    export,
    export_to = "../../frontend/types/generated/create_room_dto.ts"
)]
pub struct CreateRoomRequest {
    pub name: String,
    // slugは任意。なければ自動生成
    pub slug: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // SeaORM で接続
    let conn = Database::connect(&database_url)
        .await
        .expect("Failed to connect to DB");

    println!("Connection to the database is successful (SeaORM)");

    let state = AppState { conn };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/hello", get(hello_handler))
        .route("/api/me", get(get_me_handler))
        .route("/api/room/create", post(create_room_handler))
        .layer(cors)
        .with_state(state);

    let port = std::env::var("BACKEND_PORT")
        .unwrap_or_else(|_| "13964".to_string())
        .parse::<u16>()
        .expect("Port is not integer");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("🚀 Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_handler() -> Json<String> {
    Json("Hello from Rust & SeaORM! 🦀".to_string())
}

async fn get_me_handler(AuthUser(claims): AuthUser) -> Json<String> {
    Json(format!("Auth: {}", claims.sub))
}

/// ルーム作成ハンドラ
async fn create_room_handler(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(payload): Json<CreateRoomRequest>,
) -> Result<Json<room::Model>, (axum::http::StatusCode, String)> {
    // 1. まずユーザーを同期 (Upsert) して UserId を取得
    // Transaction を使ってアトミックにやるのも良いが、今回はシンプルに実行
    let user_id = sync_user(&state.conn, &claims)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 2. Slug の決定 (指定がなければランダム生成)
    // TODO: バリデーション
    let slug = payload.slug.unwrap_or_else(generate_random_slug);

    // 3. Room の作成 (ActiveModel を使用)
    let new_room = room::ActiveModel {
        id: Set(room::RoomId(uuid::Uuid::now_v7())), // Rust側で UUID v7 生成
        slug: Set(slug),
        name: Set(payload.name),
        owner_id: Set(user_id), // 型安全！ user::UserId型しか入らない
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };

    let room = new_room
        .insert(&state.conn)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(room))
}

/// ユーザー情報を同期する (SeaORM版)
/// 戻り値が厳格な `user::UserId` になっていることに注目！
async fn sync_user(
    conn: &DatabaseConnection,
    claims: &auth::Claims,
) -> Result<user::UserId, sea_orm::DbErr> {
    // 1. 既存ユーザーを検索
    let existing_user = User::find()
        .filter(user::Column::FirebaseUid.eq(&claims.sub))
        .one(conn)
        .await?;

    if let Some(user) = existing_user {
        // 2. 更新 (Update)
        let mut active: user::ActiveModel = user.into();
        active.email = Set(claims.email.clone());
        active.display_name = Set(claims.name.clone());
        active.photo_url = Set(claims.picture.clone());
        active.updated_at = Set(chrono::Utc::now().into());

        let updated = active.update(conn).await?;
        Ok(updated.id)
    } else {
        // 3. 新規作成 (Insert)
        let new_user = user::ActiveModel {
            id: Set(user::UserId(uuid::Uuid::now_v7())),
            firebase_uid: Set(claims.sub.clone()),
            email: Set(claims.email.clone()),
            display_name: Set(claims.name.clone()),
            photo_url: Set(claims.picture.clone()),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        };

        let inserted = new_user.insert(conn).await?;
        Ok(inserted.id)
    }
}

/// 8文字のランダムなSlugを生成するヘルパー
fn generate_random_slug() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*; // main.rs内の CreateRoomRequest などを読み込む
    use crate::entities::room::{Model as Room, RoomId};
    use crate::entities::user::{Model as User, UserId};
    use ts_rs::TS;

    #[test]
    fn export_bindings() {
        // ディレクトリが存在することを確認
        let _ = std::fs::create_dir_all("../frontend/types/generated");

        // 1. SeaORMのエンティティをエクスポート (rename指定済)
        User::export().expect("Failed to export User");
        Room::export().expect("Failed to export Room");

        // 2. Branded Types (NewType) をエクスポート
        UserId::export().expect("Failed to export UserId");
        RoomId::export().expect("Failed to export RoomId");

        // 3. APIのリクエストDTOをエクスポート
        CreateRoomRequest::export().expect("Failed to export CreateRoomRequest");

        println!("✨ TypeScript bindings updated securely!");
    }
}
