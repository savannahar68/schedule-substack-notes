use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    middleware::auth::AuthUser,
    models::ScheduledNote,
    services::substack::text_to_body_json,
    state::AppState,
};

#[derive(Deserialize)]
pub struct ScheduleNoteRequest {
    pub text: String,
    pub scheduled_at: String,
}

#[derive(Deserialize)]
pub struct UpdateNoteRequest {
    pub text: Option<String>,
    pub scheduled_at: Option<String>,
}

#[derive(Serialize)]
pub struct NoteResponse {
    pub id: String,
    pub text: String,
    pub scheduled_at: String,
    pub status: String,
    pub substack_id: Option<String>,
    pub substack_url: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
}

impl From<ScheduledNote> for NoteResponse {
    fn from(n: ScheduledNote) -> Self {
        Self {
            id: n.id,
            text: n.text,
            scheduled_at: n.scheduled_at,
            status: n.status,
            substack_id: n.substack_id,
            substack_url: n.substack_url,
            error: n.error,
            created_at: n.created_at,
        }
    }
}

pub async fn schedule(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Json(body): Json<ScheduleNoteRequest>,
) -> Result<(StatusCode, Json<NoteResponse>), AppError> {
    if body.text.trim().is_empty() {
        return Err(AppError::bad_request("Note text cannot be empty"));
    }

    let parsed = chrono::DateTime::parse_from_rfc3339(&body.scheduled_at)
        .map_err(|_| AppError::bad_request("scheduled_at must be a valid ISO 8601 datetime"))?;

    if parsed <= chrono::Utc::now() {
        return Err(AppError::bad_request("scheduled_at must be in the future"));
    }

    let body_json = text_to_body_json(&body.text);
    let body_json_str = serde_json::to_string(&body_json)
        .map_err(|e| AppError::internal(format!("JSON error: {e}")))?;

    let note_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    sqlx::query(
        "INSERT INTO scheduled_notes (id, user_id, text, body_json, scheduled_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&note_id)
    .bind(&user.id)
    .bind(&body.text)
    .bind(&body_json_str)
    .bind(&body.scheduled_at)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let note: ScheduledNote = sqlx::query_as(
        "SELECT id, user_id, text, body_json, scheduled_at, status, substack_id, substack_url, error, attempts, created_at, updated_at FROM scheduled_notes WHERE id = ?"
    )
    .bind(&note_id)
    .fetch_one(&state.pool)
    .await?;

    tracing::info!("Scheduled note {} for {}", note_id, body.scheduled_at);
    Ok((StatusCode::CREATED, Json(NoteResponse::from(note))))
}

#[derive(Serialize)]
pub struct QueueResponse {
    pub notes: Vec<NoteResponse>,
}

pub async fn queue(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<QueueResponse>, AppError> {
    let notes: Vec<ScheduledNote> = sqlx::query_as(
        r#"SELECT id, user_id, text, body_json, scheduled_at, status, substack_id, substack_url, error, attempts, created_at, updated_at
           FROM scheduled_notes
           WHERE user_id = ?
             AND (
               status IN ('pending', 'publishing')
               OR (status = 'published' AND created_at >= datetime('now', '-7 days'))
               OR (status = 'failed' AND created_at >= datetime('now', '-3 days'))
             )
           ORDER BY scheduled_at ASC
           LIMIT 100"#
    )
    .bind(&user.id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(QueueResponse {
        notes: notes.into_iter().map(NoteResponse::from).collect(),
    }))
}

pub async fn delete(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(note_id): Path<String>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query(
        "DELETE FROM scheduled_notes WHERE id = ? AND user_id = ? AND status = 'pending'"
    )
    .bind(&note_id)
    .bind(&user.id)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(
            "Note not found or cannot be deleted (only pending notes can be deleted)",
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Returns pending notes that are past their scheduled time, for the extension to publish.
#[derive(Serialize)]
pub struct DueNoteResponse {
    pub id: String,
    pub body_json: String,
}

#[derive(Serialize)]
pub struct DueNotesResponse {
    pub notes: Vec<DueNoteResponse>,
}

pub async fn due(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
) -> Result<Json<DueNotesResponse>, AppError> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT id, body_json FROM scheduled_notes
           WHERE user_id = ? AND status = 'pending'
             AND scheduled_at <= strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
           ORDER BY scheduled_at ASC LIMIT 10"#,
    )
    .bind(&user.id)
    .fetch_all(&state.pool)
    .await?;

    // Mark them as 'publishing' so concurrent calls don't double-publish
    for (id, _) in &rows {
        sqlx::query(
            "UPDATE scheduled_notes SET status = 'publishing', attempts = attempts + 1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = ?"
        )
        .bind(id)
        .execute(&state.pool)
        .await?;
    }

    Ok(Json(DueNotesResponse {
        notes: rows
            .into_iter()
            .map(|(id, body_json)| DueNoteResponse { id, body_json })
            .collect(),
    }))
}

#[derive(Deserialize)]
pub struct ReportResultRequest {
    pub success: bool,
    pub substack_id: Option<String>,
    pub substack_url: Option<String>,
    pub error: Option<String>,
}

pub async fn report_result(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(note_id): Path<String>,
    Json(body): Json<ReportResultRequest>,
) -> Result<StatusCode, AppError> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    if body.success {
        sqlx::query(
            "UPDATE scheduled_notes SET status = 'published', substack_id = ?, substack_url = ?, error = NULL, updated_at = ? WHERE id = ? AND user_id = ?"
        )
        .bind(&body.substack_id)
        .bind(&body.substack_url)
        .bind(&now)
        .bind(&note_id)
        .bind(&user.id)
        .execute(&state.pool)
        .await?;
        tracing::info!("Note {} published via extension", note_id);
    } else {
        let error = body.error.as_deref().unwrap_or("Unknown error");
        // Check attempts to decide final status
        let attempts: Option<(i64,)> =
            sqlx::query_as("SELECT attempts FROM scheduled_notes WHERE id = ? AND user_id = ?")
                .bind(&note_id)
                .bind(&user.id)
                .fetch_optional(&state.pool)
                .await?;
        let attempts = attempts.map(|(a,)| a).unwrap_or(1);
        let status = if attempts >= 3 { "failed" } else { "pending" };
        sqlx::query(
            "UPDATE scheduled_notes SET status = ?, error = ?, updated_at = ? WHERE id = ? AND user_id = ?"
        )
        .bind(status)
        .bind(error)
        .bind(&now)
        .bind(&note_id)
        .bind(&user.id)
        .execute(&state.pool)
        .await?;
        tracing::warn!("Note {} failed (attempt {}): {}", note_id, attempts, error);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(note_id): Path<String>,
    Json(body): Json<UpdateNoteRequest>,
) -> Result<Json<NoteResponse>, AppError> {
    let note: ScheduledNote = sqlx::query_as(
        "SELECT id, user_id, text, body_json, scheduled_at, status, substack_id, substack_url, error, attempts, created_at, updated_at FROM scheduled_notes WHERE id = ? AND user_id = ? AND status = 'pending'"
    )
    .bind(&note_id)
    .bind(&user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found("Note not found or cannot be edited"))?;

    let new_text = body.text.as_deref().unwrap_or(&note.text);
    let new_scheduled_at = body.scheduled_at.as_deref().unwrap_or(&note.scheduled_at);

    if body.scheduled_at.is_some() {
        let parsed = chrono::DateTime::parse_from_rfc3339(new_scheduled_at)
            .map_err(|_| AppError::bad_request("scheduled_at must be a valid ISO 8601 datetime"))?;
        if parsed <= chrono::Utc::now() {
            return Err(AppError::bad_request("scheduled_at must be in the future"));
        }
    }

    let body_json_str = if body.text.is_some() {
        let body_json = text_to_body_json(new_text);
        serde_json::to_string(&body_json)
            .map_err(|e| AppError::internal(format!("JSON error: {e}")))?
    } else {
        note.body_json.clone()
    };

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    sqlx::query(
        "UPDATE scheduled_notes SET text = ?, body_json = ?, scheduled_at = ?, updated_at = ? WHERE id = ?"
    )
    .bind(new_text)
    .bind(&body_json_str)
    .bind(new_scheduled_at)
    .bind(&now)
    .bind(&note_id)
    .execute(&state.pool)
    .await?;

    let updated: ScheduledNote = sqlx::query_as(
        "SELECT id, user_id, text, body_json, scheduled_at, status, substack_id, substack_url, error, attempts, created_at, updated_at FROM scheduled_notes WHERE id = ?"
    )
    .bind(&note_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(NoteResponse::from(updated)))
}
