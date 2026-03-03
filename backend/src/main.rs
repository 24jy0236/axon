use axum::{
    extract::State,
    http::{header, Method},
    routing::{get, post},
    Json, Router,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
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

// 🌟 参加成功時にフロントエンドに返すデータ
#[derive(Serialize, TS)]
#[ts(
    export,
    export_to = "../../frontend/types/generated/join_room_response.ts"
)]
pub struct JoinRoomResponse {
    pub room: room::Model,
    pub role: entities::room_member::Role,
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
        .route("/api/room/:slug/join", post(join_room_handler))
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
    let user_id = sync_user(&state.conn, &claims)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 2. Slug の決定とバリデーション (RoomSlugを利用)
    let slug = match payload.slug {
        Some(s) => {
            // パース失敗時は 400 Bad Request を返す
            let valid_slug = room::RoomSlug::new(s)
                .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))?;
            valid_slug.as_str().to_string()
        }
        None => generate_random_slug(),
    };

    // 3. トランザクションの開始
    let txn = state
        .conn
        .begin()
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 4. Room の作成 (txn を使用)
    let new_room = room::ActiveModel {
        id: Set(room::RoomId(uuid::Uuid::now_v7())),
        slug: Set(slug),
        name: Set(payload.name),
        owner_id: Set(user_id.clone()),
        is_active: Set(true), // migrationでデフォルトtrueなので明示しなくてもOKですが念のため
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };

    let inserted_room = new_room
        .insert(&txn)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 5. room_members への追加 (作成者をTEACHERとして登録)
    // ※ entities::room_members がsea-orm-cli等で生成されている前提です
    let new_member = room_member::ActiveModel {
        room_id: Set(inserted_room.id.clone()),
        user_id: Set(user_id),
        role: Set(room_member::Role::Teacher),
        joined_at: Set(chrono::Utc::now().into()),
    };

    new_member
        .insert(&txn)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 6. トランザクションのコミット (ここで初めてDBに変更が確定する！)
    txn.commit()
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(inserted_room))
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

/// ルーム参加・検証ハンドラ
async fn join_room_handler(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> Result<Json<JoinRoomResponse>, (axum::http::StatusCode, String)> {
    // 1. ユーザーを同期して UserId を取得
    let user_id = sync_user(&state.conn, &claims)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 2. 指定された slug の部屋が存在するか確認
    let target_room = room::Entity::find()
        .filter(room::Column::Slug.eq(slug))
        .one(&state.conn)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 部屋がなければ 404
    let target_room = match target_room {
        Some(r) => r,
        None => {
            return Err((
                axum::http::StatusCode::NOT_FOUND,
                "Room not found".to_string(),
            ))
        }
    };

    // 3. 既にメンバーとして登録されているか確認
    let existing_member = entities::room_member::Entity::find()
        .filter(entities::room_member::Column::RoomId.eq(target_room.id.clone()))
        .filter(entities::room_member::Column::UserId.eq(user_id.clone()))
        .one(&state.conn)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 4. メンバー登録処理と権限の決定
    let role = if let Some(member) = existing_member {
        // 既にメンバーならその権限を返す
        member.role
    } else {
        // 初めての参加なら STUDENT として登録
        let new_member = entities::room_member::ActiveModel {
            room_id: Set(target_room.id.clone()),
            user_id: Set(user_id),
            role: Set(entities::room_member::Role::Student),
            joined_at: Set(chrono::Utc::now().into()),
        };

        new_member
            .insert(&state.conn)
            .await
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        entities::room_member::Role::Student
    };

    // 5. 部屋の情報と権限をフロントエンドに返す
    Ok(Json(JoinRoomResponse {
        room: target_room,
        role,
    }))
}

#[cfg(test)]
mod tests {
    use super::*; // main.rs内の CreateRoomRequest などを読み込む
    use crate::entities::room::{Model as Room, RoomId};
    use crate::entities::room_member::{Model as RoomMember, Role};
    use crate::entities::user::{Model as User, UserId};
    use ts_rs::TS;

    #[test]
    fn export_bindings() {
        // ディレクトリが存在することを確認
        let _ = std::fs::create_dir_all("../frontend/types/generated");

        // 1. SeaORMのエンティティをエクスポート (rename指定済)
        User::export().expect("Failed to export User");
        Room::export().expect("Failed to export Room");
        RoomMember::export().expect("Failed to export RoomMember");

        Role::export().expect("Failed to export Role");

        // 2. Branded Types (NewType) をエクスポート
        UserId::export().expect("Failed to export UserId");
        RoomId::export().expect("Failed to export RoomId");

        // 3. APIのリクエストDTOをエクスポート
        CreateRoomRequest::export().expect("Failed to export CreateRoomRequest");
        JoinRoomResponse::export().expect("Failed to export JoinRoomResponse");

        println!("✨ TypeScript bindings updated securely!");
    }
}
