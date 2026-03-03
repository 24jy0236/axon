use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email_verified: Option<bool>,
    pub exp: usize,
    pub iat: usize,
    pub aud: String,
    pub iss: String,
}

// 🌟 新しく追加：トークン文字列を受け取って検証する関数
pub fn verify_token(token: &str) -> Result<Claims, (StatusCode, String)> {
    let _header = decode_header(token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid JWT header".to_string()))?;

    let project_id = env::var("FIREBASE_PROJECT_ID").unwrap_or_default();

    let mut validation = Validation::new(Algorithm::RS256);
    validation.insecure_disable_signature_validation(); // 開発用
    validation.set_audience(&[&project_id]);

    let dummy_key = DecodingKey::from_secret(&[]);

    let token_data = decode::<Claims>(token, &dummy_key, &validation).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            format!("JWT validation failed: {}", e),
        )
    })?;

    Ok(token_data.claims)
}

pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            ))?;

        let token = if auth_header.starts_with("Bearer ") {
            &auth_header[7..]
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization header format".to_string(),
            ));
        };

        // 🌟 切り出した関数を呼び出すだけ！
        let claims = verify_token(token)?;
        Ok(AuthUser(claims))
    }
}