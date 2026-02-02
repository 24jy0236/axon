use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;
use uuid::Uuid;

// ユーザーモデル
#[derive(Debug, Serialize, Deserialize, FromRow, TS)]
#[ts(export_to = "../../frontend/types/generated/user.ts")]
pub struct User {
    pub id: Uuid,
    pub firebase_uid: String, // DBカラム名と一致させる
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub created_at: DateTime<Utc>, // DB側でNOT NULLなのでOption不要
    pub updated_at: DateTime<Utc>,
}

// ルームモデル
#[derive(Debug, Serialize, Deserialize, FromRow, TS)]
#[ts(export_to = "../../frontend/types/generated/room.ts")]
pub struct Room {
    pub id: Uuid,
    pub slug: String, // URL/招待コード用
    pub name: String,
    pub owner_id: Uuid, // DB側でNOT NULLなのでOption不要
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ルーム作成リクエスト
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export_to = "../../frontend/types/generated/create_room_dto.ts")]
pub struct CreateRoomRequest {
    pub name: String,
    pub slug: Option<String>, // 任意指定。なければ自動生成
}