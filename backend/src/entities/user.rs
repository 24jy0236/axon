use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// --- NewType Pattern ---
// これにより、UserId はただの Uuid ではなくなる。
// RoomId と取り違えるとコンパイルエラーになる。
#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../frontend/types/generated/branded_types.ts")]
pub struct UserId(pub uuid::Uuid);

impl sea_orm::TryFromU64 for UserId {
    fn try_from_u64(_: u64) -> Result<Self, sea_orm::DbErr> {
        Err(sea_orm::DbErr::Custom(
            "Cannot convert u64 to UserId (using UUID)".into(),
        ))
    }
}

// --- Entity Definition ---
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "users")]
#[ts(export, export_to = "../../frontend/types/generated/user.ts", rename = "User")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: UserId, // ここで NewType を使用！
    #[sea_orm(unique)]
    pub firebase_uid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // リレーションがあればここに書く（今回は省略）
}

impl ActiveModelBehavior for ActiveModel {}
