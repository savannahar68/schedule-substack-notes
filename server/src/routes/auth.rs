use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    middleware::auth::AuthUser,
    services::{encryption, substack, substack::SubstackCookies},
    state::AppState,
};

#[derive(Deserialize)]
pub struct RegisterCookiesRequest {
    pub cookies: CookiesPayload,
    pub handle: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CookiesPayload {
    pub connect_sid: Option<String>,
    pub substack_sid: Option<String>,
    pub substack_lli: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterCookiesResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub handle: Option<String>,
}

pub async fn register_cookies(
    State(state): State<AppState>,
    Json(body): Json<RegisterCookiesRequest>,
) -> Result<Json<RegisterCookiesResponse>, AppError> {
    let cookies = SubstackCookies {
        connect_sid: body.cookies.connect_sid,
        substack_sid: body.cookies.substack_sid,
        substack_lli: body.cookies.substack_lli,
    };

    // Use the handle provided by the extension (fetched browser-side, bypassing Cloudflare).
    // Fall back to a server-side verify only if no handle was provided.
    let handle = if body.handle.is_some() {
        body.handle.clone()
    } else {
        match substack::verify_cookies(&cookies, &state.http_client).await {
            Ok(user) => user.handle,
            Err(e) => {
                tracing::warn!("Cookie verification returned an error (continuing anyway): {e}");
                None
            }
        }
    };

    // Serialize cookies for encryption
    let cookies_json = serde_json::to_string(&cookies)
        .map_err(|e| AppError::internal(format!("Serialization error: {e}")))?;

    let (encrypted_cookies, cookie_iv) = encryption::encrypt(&cookies_json, &state.key)
        .map_err(|e| AppError::internal(format!("Encryption error: {e}")))?;

    let new_token = format!("usr_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // Upsert: if user with this handle exists, update their cookies+token
    let existing: Option<(String,)> = if let Some(ref h) = handle {
        sqlx::query_as("SELECT id FROM users WHERE substack_handle = ?")
            .bind(h)
            .fetch_optional(&state.pool)
            .await?
    } else {
        None
    };

    let final_token = if let Some((existing_id,)) = existing {
        sqlx::query(
            "UPDATE users SET encrypted_cookies = ?, cookie_iv = ?, auth_token = ?, cookies_valid_at = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&encrypted_cookies)
        .bind(&cookie_iv)
        .bind(&new_token)
        .bind(&now)
        .bind(&now)
        .bind(&existing_id)
        .execute(&state.pool)
        .await?;
        new_token
    } else {
        let user_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO users (id, encrypted_cookies, cookie_iv, auth_token, substack_handle, cookies_valid_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&user_id)
        .bind(&encrypted_cookies)
        .bind(&cookie_iv)
        .bind(&new_token)
        .bind(&handle)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&state.pool)
        .await?;
        new_token
    };

    tracing::info!("Registered cookies for user {:?}", handle);

    Ok(Json(RegisterCookiesResponse {
        token: final_token,
        user: UserInfo { handle },
    }))
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub valid: bool,
    pub handle: Option<String>,
    pub last_checked: Option<String>,
}

pub async fn health(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>, AppError> {
    let cookies_json = encryption::decrypt(&user.encrypted_cookies, &user.cookie_iv, &state.key)
        .map_err(|e| AppError::internal(format!("Decryption error: {e}")))?;

    let cookies: SubstackCookies = serde_json::from_str(&cookies_json)
        .map_err(|e| AppError::internal(format!("Cookie parse error: {e}")))?;

    match substack::verify_cookies(&cookies, &state.http_client).await {
        Ok(_) => {
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            sqlx::query("UPDATE users SET cookies_valid_at = ?, updated_at = ? WHERE id = ?")
                .bind(&now)
                .bind(&now)
                .bind(&user.id)
                .execute(&state.pool)
                .await?;

            Ok(Json(HealthResponse {
                valid: true,
                handle: user.substack_handle,
                last_checked: Some(now),
            }))
        }
        Err(_) => Ok(Json(HealthResponse {
            valid: false,
            handle: user.substack_handle,
            last_checked: user.cookies_valid_at,
        })),
    }
}
