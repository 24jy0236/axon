use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{header, Method},
    response::Response,
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

use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

pub struct WsState {
    pub rooms: Mutex<HashMap<room::RoomId, broadcast::Sender<String>>>,
}

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection, // Pool<Postgres> ではなく SeaORM のコネクション
    ws_state: Arc<WsState>,
}

#[derive(Deserialize)]
pub struct WsQuery {
    token: String,
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

// 🌟 リアルタイムチャットでやり取りされるメッセージの型
#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "../../frontend/types/generated/ws_message.ts")]
pub struct WsMessagePayload {
    pub id: String, // UUIDを文字列として送る
    pub content: String,
    pub sender_name: String,
    pub sender_photo_url: Option<String>,
    pub sender_role: entities::room_member::Role,
    pub sent_at: String,
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

    // 🌟 WebSocket用のステートを初期化
    let ws_state = Arc::new(WsState {
        rooms: Mutex::new(HashMap::new()),
    });

    // 🌟 状態をまとめる
    let state = AppState { conn, ws_state };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_origin(Any);

    let app = Router::new()
        .route("/api/hello", get(hello_handler))
        .route("/api/me", get(get_me_handler))
        .route("/api/room/create", post(create_room_handler))
        .route("/api/room/{slug}/join", post(join_room_handler))
        .route("/api/room/{slug}/ws", get(ws_handler))
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

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(slug): Path<String>,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
) -> Result<Response, (axum::http::StatusCode, String)> {
    // 1. クエリパラメータのトークンを検証
    let claims = auth::verify_token(&query.token)?;

    // 2. ユーザーを同期して UserId を取得 (のちほどメッセージ送信者を特定するため)
    let user_id = sync_user(&state.conn, &claims)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 3. (オプション) 該当の部屋が存在するか、ユーザーが参加メンバーか確認する処理を入れるとより安全です
    // 今回は一旦スキップし、接続を許可します

    // 4. WebSocketのコネクションにアップグレード
    // アップグレードが成功したら `handle_socket` という非同期タスクに処理を移譲します
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, slug, user_id)))
}

async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    slug: String,
    user_id: entities::user::UserId,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // 1. ルーム情報の取得
    let target_room = match room::Entity::find()
        .filter(room::Column::Slug.eq(&slug))
        .one(&state.conn)
        .await
    {
        Ok(Some(r)) => r,
        _ => return,
    };
    let room_id = target_room.id;

    // 🌟 2. 接続してきたユーザーの情報と権限をDBから取得しておく！
    let current_user = user::Entity::find_by_id(user_id.clone())
        .one(&state.conn)
        .await
        .unwrap()
        .unwrap(); // 実際の運用ではエラーハンドリング推奨

    let current_member = entities::room_member::Entity::find()
        .filter(entities::room_member::Column::RoomId.eq(room_id.clone()))
        .filter(entities::room_member::Column::UserId.eq(user_id.clone()))
        .one(&state.conn)
        .await
        .unwrap()
        .unwrap();

    let rx = {
        let mut rooms = state.ws_state.rooms.lock().await;
        let tx = rooms.entry(room_id.clone()).or_insert_with(|| {
            let (tx, _rx) = broadcast::channel(100);
            tx
        });
        tx.subscribe()
    };

    // 【送信タスク】変更なし (Stringとして送られてきたJSONをそのままブラウザに流すだけ)
    let mut send_task = tokio::spawn(async move {
        let mut rx = rx;
        while let Ok(msg) = rx.recv().await {
            if ws_sender
                .send(axum::extract::ws::Message::Text(msg.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // 🌟 【受信タスク】JSONを組み立てて配信するように変更
    let state_clone = state.clone();
    let room_id_clone = room_id.clone();
    let user_id_clone = user_id.clone();

    // クロージャに値をMoveさせるためのクローン
    let sender_name = current_user
        .display_name
        .unwrap_or_else(|| "名無し".to_string());
    let sender_photo_url = current_user.photo_url;
    let sender_role = current_member.role;

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                let text_str = text.to_string();
                let message_id = uuid::Uuid::now_v7();

                // 1. DBに保存
                let new_message = entities::message::ActiveModel {
                    id: Set(entities::message::MessageId(message_id.clone())),
                    room_id: Set(room_id_clone.clone()),
                    sender_id: Set(user_id_clone.clone()),
                    content: Set(text_str.clone()),
                    is_dm: Set(false),
                    sent_at: Set(chrono::Utc::now().into()),
                    ..Default::default()
                };

                if let Err(e) = new_message.insert(&state_clone.conn).await {
                    eprintln!("Failed to save message to DB: {}", e);
                    continue;
                }

                // 🌟 2. フロントエンドに送るJSONペイロードを作成
                let payload = WsMessagePayload {
                    id: message_id.to_string(),
                    content: text_str,
                    sender_name: sender_name.clone(),
                    sender_photo_url: sender_photo_url.clone(),
                    sender_role: sender_role.clone(),
                    sent_at: chrono::Utc::now().to_rfc3339(),
                };

                // JSON文字列に変換
                if let Ok(json_string) = serde_json::to_string(&payload) {
                    // ルームの全員にJSONを配信
                    let rooms = state_clone.ws_state.rooms.lock().await;
                    if let Some(tx) = rooms.get(&room_id_clone) {
                        let _ = tx.send(json_string);
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    println!("👋 User {:?} disconnected from room: {}", user_id, slug);
}

#[cfg(test)]
mod tests {
    use super::*; // main.rs内の CreateRoomRequest などを読み込む
    use crate::entities::message::{MessageId, Model as Message};
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
        Message::export().expect("Failed to export RoomMember");

        Role::export().expect("Failed to export Role");

        // 2. Branded Types (NewType) をエクスポート
        UserId::export().expect("Failed to export UserId");
        RoomId::export().expect("Failed to export RoomId");
        MessageId::export().expect("Failed to export MessageId");

        // 3. APIのリクエストDTOをエクスポート
        CreateRoomRequest::export().expect("Failed to export CreateRoomRequest");
        JoinRoomResponse::export().expect("Failed to export JoinRoomResponse");
        WsMessagePayload::export().expect("Failed to export WsMessagePayload");

        println!("✨ TypeScript bindings updated securely!");
    }
}
