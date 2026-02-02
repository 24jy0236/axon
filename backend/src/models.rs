use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ts_rs::TS;
use uuid::Uuid;

// ユーザーモデル
// Frontendからも参照されるので #[derive(TS)] をつける
// #[ts(export)] をつけると、ビルド時に自動でTSファイルが書き出される設定もできるが、
// 今回はテスト実行時に一括生成する方式をとる（制御しやすいから）。
#[derive(Debug, Serialize, Deserialize, FromRow, TS)]
#[ts(export_to = "../../frontend/types/generated/user.ts")] // 出力先を指定！
pub struct User {
    pub id: Uuid,
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    // chronoのDateTimeもTS側ではstringとして扱われる（設定でDateにもできる）
    pub created_at: Option<DateTime<Utc>>,
}

// ルームモデル
#[derive(Debug, Serialize, Deserialize, FromRow, TS)]
#[ts(export_to = "../../frontend/types/generated/room.ts")]
pub struct Room {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Option<Uuid>,
    pub created_at: Option<DateTime<Utc>>,
}

// APIのリクエスト/レスポンス用DTOも定義しておくと最高だ
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(export_to = "../../frontend/types/generated/create_room_dto.ts")]
pub struct CreateRoomRequest {
    pub name: String,
}
