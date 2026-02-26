use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use super::user::UserId; // UserId型をインポート

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/generated/branded_types.ts")]
pub struct RoomId(pub uuid::Uuid);

impl sea_orm::TryFromU64 for RoomId {
    fn try_from_u64(_: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::Custom(
            "Cannot convert u64 to RoomId (using UUID)".into(),
        ))
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "rooms")]
#[ts(export, export_to = "../../frontend/types/generated/room.ts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: RoomId,
    #[sea_orm(unique)]
    pub slug: String,
    pub name: String,
    pub owner_id: UserId, // ここも厳格に UserId 型！
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
