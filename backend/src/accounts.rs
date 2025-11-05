use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use sqlx::{PgPool, Row};

use crate::models::{
    CreateAccountRequest, CreateAccountResponse, AccountResponse,
    UpdateAccountRequest, UpdateAccountResponse,
    DeleteAccountResponse, Account,
};
use crate::auth::AppError;
use crate::middleware;

pub async fn create_account(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<CreateAccountResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    if req.name.is_empty() || req.name.len() > 50 {
        return Err(AppError::BadRequest("Account name must be between 1 and 50 characters".to_string()));
    }

    let currency = req.currency.unwrap_or_else(|| "USD".to_string());

    // Use account name as type (users can create custom account names)
    let row = sqlx::query(
        "INSERT INTO accounts (user_id, name, type, currency) VALUES ($1, $2, $2, $3) RETURNING id, user_id, name, currency, balance::text, created_at"
    )
    .bind(auth.user_id)
    .bind(&req.name)
    .bind(&currency)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let account = Account {
        id: row.get(0),
        user_id: row.get(1),
        name: row.get(2),
        currency: row.get(3),
        balance: row.get(4),
        created_at: row.get(5),
    };

    Ok(Json(CreateAccountResponse {
        message: "Account created successfully".to_string(),
        account: account_to_response(account),
    }))
}

pub async fn get_accounts(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<AccountResponse>>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    let rows = sqlx::query(
        "SELECT id, user_id, name, currency, balance::text, created_at FROM accounts WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(auth.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let accounts: Vec<Account> = rows
        .into_iter()
        .map(|row| Account {
            id: row.get(0),
            user_id: row.get(1),
            name: row.get(2),
            currency: row.get(3),
            balance: row.get(4),
            created_at: row.get(5),
        })
        .collect();

    let accounts_response: Vec<AccountResponse> = accounts
        .into_iter()
        .map(account_to_response)
        .collect();

    Ok(Json(accounts_response))
}

pub async fn update_account(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    axum::extract::Path(account_id): axum::extract::Path<i32>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<Json<UpdateAccountResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    if req.name.is_empty() || req.name.len() > 50 {
        return Err(AppError::BadRequest("Account name must be between 1 and 50 characters".to_string()));
    }

    let row = sqlx::query(
        "UPDATE accounts SET name = $1, type = $1 WHERE id = $2 AND user_id = $3 RETURNING id, user_id, name, currency, balance::text, created_at"
    )
    .bind(&req.name)
    .bind(account_id)
    .bind(auth.user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let row = match row {
        Some(r) => r,
        None => return Err(AppError::BadRequest("Account not found or you don't have permission to update it".to_string())),
    };

    let account = Account {
        id: row.get(0),
        user_id: row.get(1),
        name: row.get(2),
        currency: row.get(3),
        balance: row.get(4),
        created_at: row.get(5),
    };

    Ok(Json(UpdateAccountResponse {
        message: "Account updated successfully".to_string(),
        account: account_to_response(account),
    }))
}

pub async fn delete_account(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    axum::extract::Path(account_id): axum::extract::Path<i32>,
) -> Result<Json<DeleteAccountResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    let result = sqlx::query(
        "DELETE FROM accounts WHERE id = $1 AND user_id = $2"
    )
    .bind(account_id)
    .bind(auth.user_id)
    .execute(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(AppError::BadRequest("Account not found or you don't have permission to delete it".to_string()));
    }

    Ok(Json(DeleteAccountResponse {
        message: "Account deleted successfully".to_string(),
    }))
}


fn account_to_response(account: Account) -> AccountResponse {
    AccountResponse {
        id: account.id,
        name: account.name,
        currency: account.currency,
        balance: account.balance,
        created_at: account.created_at,
    }
}

