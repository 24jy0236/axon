use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Claims {
    /// Firebase UID (UserInfo.uid に相当)
    pub sub: String,
    /// メールアドレス
    pub email: Option<String>,
    /// 表示名 (UserInfo.displayName に相当)
    pub name: Option<String>,
    /// プロフィール画像URL (UserInfo.photoURL に相当)
    pub picture: Option<String>,
    /// メール確認済みか
    pub email_verified: Option<bool>,
    /// 有効期限 (必須)
    pub exp: usize,
    /// 発行時刻
    pub iat: usize,
    /// プロジェクトID
    pub aud: String,
    /// 発行者 (https://securetoken.google.com/<project_id>)
    pub iss: String,
}

pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Authorizationヘッダーを取得
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            ))?;

        // 2. "Bearer " プレフィックスを除去
        let token = if auth_header.starts_with("Bearer ") {
            &auth_header[7..]
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization header format".to_string(),
            ));
        };

        // 3. JWTのヘッダーをデコードして、キーID (kid) を取得
        // (本来はここでGoogleのJWKエンドポイントから鍵を取得して検証するけど、
        //  今回はまず「トークンの形式チェック」と「FirebaseプロジェクトIDの一致」まで実装するよ！)

        let header = decode_header(token)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid JWT header".to_string()))?;

        // ※ ここでGoogleの公開鍵リストから header.kid に対応する鍵を探すのが本来のフロー。
        //    今回は開発用として「検証をスキップするモード」でデコードだけしてみる。
        //    (セキュリティ的にはNGだけど、疎通確認には最適！後で強化しよう)

        // --- 簡易実装版 (検証なしデコード) ---
        let project_id = env::var("FIREBASE_PROJECT_ID").unwrap_or_default();

        let mut validation = Validation::new(Algorithm::RS256);
        validation.insecure_disable_signature_validation(); // ★開発用：署名検証を無効化 TODO:
        validation.set_audience(&[project_id]); // プロジェクトIDのチェックはする！

        // ダミーのキーを使ってデコード（署名検証しないのでキーは何でもいい）
        let dummy_key = DecodingKey::from_secret(&[]);

        let token_data = decode::<Claims>(token, &dummy_key, &validation).map_err(|e| {
            (
                StatusCode::UNAUTHORIZED,
                format!("JWT validation failed: {}", e),
            )
        })?;

        // 成功！クレームを詰めて返す
        Ok(AuthUser(token_data.claims))
    }
}
