use backend::models::*; // backend クレートの models をインポート
use ts_rs::TS;

#[test]
fn export_bindings() {
    // ディレクトリが存在することを確認（念のため）
    let _ = std::fs::create_dir_all("../frontend/types/generated");

    // 型をエクスポート
    User::export().expect("Failed to export User");
    Room::export().expect("Failed to export Room");
    CreateRoomRequest::export().expect("Failed to export CreateRoomRequest");
    
    println!("✨ TypeScript bindings updated!");
}