use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use serde_json::json;

use crate::{models::User, state::AppState};

pub struct AuthUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = if let Some(t) = auth_header.strip_prefix("Bearer ") {
            t.trim()
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Missing or invalid Authorization header" })),
            ));
        };

        let user: Option<User> = sqlx::query_as(
            "SELECT id, encrypted_cookies, cookie_iv, auth_token, substack_handle, cookies_valid_at, created_at, updated_at FROM users WHERE auth_token = ?"
        )
        .bind(token)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("DB error in auth middleware: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        })?;

        match user {
            Some(u) => Ok(AuthUser(u)),
            None => Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Invalid token" })),
            )),
        }
    }
}
