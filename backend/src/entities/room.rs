use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use super::user::UserId; // UserId型をインポート

#[derive(Clone, Debug, Hash, PartialEq, Eq, DeriveValueType, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/generated/branded_types.ts")]
pub struct RoomId(pub uuid::Uuid);

impl sea_orm::TryFromU64 for RoomId {
    fn try_from_u64(_: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::Custom(
            "Cannot convert u64 to RoomId (using UUID)".into(),
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoomSlug(String);

impl RoomSlug {
    /// 文字列を受け取り、バリデーションと正規化（小文字化）を行ってから RoomSlug を返す
    pub fn new(slug: String) -> Result<Self, &'static str> {
        let len = slug.chars().count();
        if len < 4 || len > 16 {
            return Err("Slug must be between 4 and 16 characters");
        }

        if !slug.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err("Slug must be alphanumeric");
        }

        // 成功したら小文字に変換してラップする
        Ok(Self(slug.to_lowercase()))
    }

    /// 内部の文字列を取り出すためのヘルパー
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "rooms")]
#[ts(export, export_to = "../../frontend/types/generated/room.ts", rename = "Room")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: RoomId,
    #[sea_orm(unique)]
    pub slug: String,
    pub name: String,
    pub owner_id: UserId, // ここも厳格に UserId 型！
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::OwnerId",
        to = "super::user::Column::Id"
    )]
    Owner,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Owner.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
