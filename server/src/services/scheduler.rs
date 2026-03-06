use std::time::Duration;

use reqwest::Client;
use sqlx::SqlitePool;

use crate::services::{encryption, substack};

pub async fn run(pool: SqlitePool, key: [u8; 32], client: Client) {
    tracing::info!("Scheduler started, polling every 30 seconds");
    loop {
        if let Err(e) = tick(&pool, &key, &client).await {
            tracing::error!("Scheduler tick error: {e:#}");
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

struct DueNote {
    id: String,
    attempts: i64,
    encrypted_cookies: String,
    cookie_iv: String,
    body_json: String,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for DueNote {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Self {
            id: row.try_get("id")?,
            attempts: row.try_get("attempts")?,
            encrypted_cookies: row.try_get("encrypted_cookies")?,
            cookie_iv: row.try_get("cookie_iv")?,
            body_json: row.try_get("body_json")?,
        })
    }
}

async fn tick(pool: &SqlitePool, key: &[u8; 32], client: &Client) -> anyhow::Result<()> {
    let rows: Vec<DueNote> = sqlx::query_as(
        r#"
        SELECT
            sn.id,
            sn.attempts,
            u.encrypted_cookies,
            u.cookie_iv,
            sn.body_json
        FROM scheduled_notes sn
        JOIN users u ON u.id = sn.user_id
        WHERE sn.scheduled_at <= strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
          AND sn.status = 'pending'
        LIMIT 50
        "#
    )
    .fetch_all(pool)
    .await?;

    if !rows.is_empty() {
        tracing::info!("Processing {} due notes", rows.len());
    }

    for row in rows {
        let id = row.id.clone();
        if let Err(e) = process_note(pool, key, client, row).await {
            tracing::error!("Failed to process note {id}: {e:#}");
        }
    }

    Ok(())
}

async fn process_note(
    pool: &SqlitePool,
    key: &[u8; 32],
    client: &Client,
    note: DueNote,
) -> anyhow::Result<()> {
    let id = &note.id;

    // Mark as publishing to prevent double-processing
    sqlx::query(
        "UPDATE scheduled_notes SET status = 'publishing', attempts = attempts + 1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?"
    )
    .bind(id)
    .execute(pool)
    .await?;

    // Decrypt cookies
    let cookies_json = match encryption::decrypt(&note.encrypted_cookies, &note.cookie_iv, key) {
        Ok(j) => j,
        Err(e) => {
            mark_failed(pool, id, &format!("Cookie decryption failed: {e}"), note.attempts).await?;
            return Ok(());
        }
    };

    let cookies: substack::SubstackCookies = match serde_json::from_str(&cookies_json) {
        Ok(c) => c,
        Err(e) => {
            mark_failed(pool, id, &format!("Cookie parse failed: {e}"), note.attempts).await?;
            return Ok(());
        }
    };

    let body_json: serde_json::Value = match serde_json::from_str(&note.body_json) {
        Ok(v) => v,
        Err(e) => {
            mark_failed(pool, id, &format!("body_json parse failed: {e}"), note.attempts).await?;
            return Ok(());
        }
    };

    match substack::publish_note(&cookies, &body_json, client).await {
        Ok(published) => {
            let substack_id = published.id.map(|i| i.to_string());
            let substack_url = published.url;
            sqlx::query(
                "UPDATE scheduled_notes SET status = 'published', substack_id = ?, substack_url = ?, error = NULL, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?"
            )
            .bind(&substack_id)
            .bind(&substack_url)
            .bind(id)
            .execute(pool)
            .await?;
            tracing::info!("Published note {id}");
        }
        Err(e) => {
            mark_failed(pool, id, &e.to_string(), note.attempts + 1).await?;
        }
    }

    Ok(())
}

async fn mark_failed(pool: &SqlitePool, id: &str, error: &str, attempts: i64) -> anyhow::Result<()> {
    let status = if attempts >= 3 { "failed" } else { "pending" };
    tracing::warn!("Note {id} -> status={status} after {attempts} attempts: {error}");
    sqlx::query(
        "UPDATE scheduled_notes SET status = ?, error = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?"
    )
    .bind(status)
    .bind(error)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
