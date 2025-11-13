use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Duration, Utc};

use crate::models::{
    RegisterRequest, RegisterResponse, LoginRequest, LoginResponse,
    LogoutResponse, ErrorResponse, User,
};
use crate::middleware;

pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    if req.username.is_empty() || req.username.len() > 50 {
        return Err(AppError::BadRequest("Username must be between 1 and 50 characters".to_string()));
    }
    if req.password.is_empty() || req.password.len() < 6 {
        return Err(AppError::BadRequest("Password must be at least 6 characters".to_string()));
    }

    let password_hash = hash_password(&req.password)?;

    let user_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id"
    )
    .bind(&req.username)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.constraint() == Some("users_username_key") {
                return AppError::BadRequest("Username already exists".to_string());
            }
        }
        AppError::InternalServerError(format!("Database error: {}", e))
    })?;

    // Create default accounts for the new user
    let default_accounts = ["Checking", "Savings"];
    for account_name in &default_accounts {
        sqlx::query(
            "INSERT INTO accounts (user_id, name, type, currency) VALUES ($1, $2, $2, 'USD')"
        )
        .bind(user_id)
        .bind(*account_name)
        .execute(&pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create default account: {}", e)))?;
    }

    // Create default categories for the new user
    let default_categories = ["Salary", "Food", "Rent", "Travel"];
    for category_name in &default_categories {
        sqlx::query(
            "INSERT INTO categories (user_id, name) VALUES ($1, $2)"
        )
        .bind(user_id)
        .bind(*category_name)
        .execute(&pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create default category: {}", e)))?;
    }

    Ok(Json(RegisterResponse {
        message: "User registered successfully".to_string(),
        user_id,
    }))
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, password_hash, created_at FROM users WHERE username = $1"
    )
    .bind(&req.username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let user = match user {
        Some(u) => u,
        None => return Err(AppError::Unauthorized("Invalid username or password".to_string())),
    };

    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("Invalid username or password".to_string()));
    }

    let token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(24);

    sqlx::query(
        "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3)"
    )
    .bind(user.id)
    .bind(&token)
    .bind(expires_at)
    .execute(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(Json(LoginResponse {
        message: "Login successful".to_string(),
        token,
        user_id: user.id,
    }))
}

pub async fn logout(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<LogoutResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;
    
    sqlx::query(
        "UPDATE sessions SET is_valid = FALSE WHERE token = $1"
    )
    .bind(&auth.token)
    .execute(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(Json(LogoutResponse {
        message: "Logout successful".to_string(),
    }))
}

fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::InternalServerError(format!("Password hashing error: {}", e)))?;

    Ok(password_hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::InternalServerError(format!("Invalid hash format: {}", e)))?;

    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AppError::InternalServerError(format!("Verification error: {}", e))),
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthUser {
    pub user_id: i32,
    pub token: String,
}

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::InternalServerError(msg) => {
                eprintln!("Internal server error: {}", msg);
                let is_dev = std::env::var("RUST_ENV").unwrap_or_default() == "development";
                let response_msg = if is_dev {
                    msg
                } else {
                    "Internal server error".to_string()
                };
                (StatusCode::INTERNAL_SERVER_ERROR, response_msg)
            }
        };

        (status, Json(ErrorResponse { error: error_message })).into_response()
    }
}

