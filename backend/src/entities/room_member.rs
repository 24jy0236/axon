use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::room::RoomId;
use super::user::UserId;

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, TS)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(export, export_to = "../../frontend/types/generated/role.ts")]
pub enum Role {
    #[sea_orm(string_value = "TEACHER")]
    Teacher,
    #[sea_orm(string_value = "STUDENT")]
    Student,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "room_members")]
#[ts(export, export_to = "../../frontend/types/generated/room_member.ts", rename = "RoomMember")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub room_id: RoomId,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: UserId,
    pub role: Role, // 'TEACHER' または 'STUDENT'
    pub joined_at: DateTimeWithTimeZone,
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
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

// Roomとのリレーション
impl Related<super::room::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Room.def()
    }
}

// Userとのリレーション
impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}