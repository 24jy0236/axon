use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::room::RoomId;
use super::user::UserId;

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/generated/branded_types.ts")]
pub struct MessageId(pub uuid::Uuid);

impl sea_orm::TryFromU64 for MessageId {
    fn try_from_u64(_: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::Custom(
            "Cannot convert u64 to MessageId (using UUID)".into(),
        ))
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "messages")]
#[ts(export, export_to = "../../frontend/types/generated/message.ts", rename = "Message")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: MessageId,
    pub room_id: RoomId,
    pub sender_id: UserId,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub recipient_id: Option<UserId>, // DM用の宛先 (null許容)
    pub is_dm: bool,
    pub sent_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::room::Entity",
        from = "Column::RoomId",
        to = "super::room::Column::Id"
    )]
    Room,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::SenderId",
        to = "super::user::Column::Id"
    )]
    Sender,
}

// Roomとのリレーション
impl Related<super::room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Room.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}