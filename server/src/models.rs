use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub encrypted_cookies: String,
    pub cookie_iv: String,
    pub auth_token: String,
    pub substack_handle: Option<String>,
    pub cookies_valid_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct ScheduledNote {
    pub id: String,
    pub user_id: String,
    pub text: String,
    pub body_json: String,
    pub scheduled_at: String,
    pub status: String,
    pub substack_id: Option<String>,
    pub substack_url: Option<String>,
    pub error: Option<String>,
    pub attempts: i64,
    pub created_at: String,
    pub updated_at: String,
}
