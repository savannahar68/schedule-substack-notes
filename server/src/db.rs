use anyhow::Result;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use crate::config::Config;

pub async fn setup(config: &Config) -> Result<SqlitePool> {
    // Ensure data directory exists
    std::fs::create_dir_all(&config.data_dir)?;

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url())
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    tracing::info!("Database ready at {}/scheduler.db", config.data_dir.display());
    Ok(pool)
}
