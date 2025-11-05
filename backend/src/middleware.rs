use axum::http::HeaderMap;
use sqlx::PgPool;
use crate::auth::{AuthUser, AppError};

pub async fn verify_auth(
    pool: &PgPool,
    headers: &HeaderMap,
) -> Result<AuthUser, AppError> {
    let token = extract_token(headers)
        .ok_or_else(|| AppError::Unauthorized("Missing or invalid authorization header".to_string()))?;

    let session = sqlx::query_as::<_, (i32, String, bool, chrono::DateTime<chrono::Utc>)>(
        "SELECT user_id, token, is_valid, expires_at FROM sessions WHERE token = $1"
    )
    .bind(&token)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let (user_id, token, is_valid, expires_at) = match session {
        Some(s) => s,
        None => return Err(AppError::Unauthorized("Invalid session token".to_string())),
    };

    if !is_valid {
        return Err(AppError::Unauthorized("Session has been invalidated".to_string()));
    }

    if expires_at < chrono::Utc::now() {
        return Err(AppError::Unauthorized("Session has expired".to_string()));
    }

    Ok(AuthUser { user_id, token })
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("Authorization")?.to_str().ok()?;
    
    if auth_header.starts_with("Bearer ") {
        Some(auth_header[7..].to_string())
    } else {
        Some(auth_header.to_string())
    }
}
