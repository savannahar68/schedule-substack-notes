use reqwest::Client;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub key: [u8; 32],
    pub http_client: Client,
}
