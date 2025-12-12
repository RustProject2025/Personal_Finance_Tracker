use axum::{
    extract::{State, Query},
    http::HeaderMap,
    Json,
};
use sqlx::{PgPool, Row};
use serde::Deserialize;
use chrono::NaiveDate;

use crate::models::{
    CreateTransactionRequest, CreateTransactionResponse, TransactionResponse,
    TransferRequest, TransferResponse,
};
use crate::auth::AppError;
use crate::middleware;

#[derive(Debug, Deserialize)]
pub struct GetTransactionsQuery {
    pub account_id: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

pub async fn create_transaction(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<Json<CreateTransactionResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    // Determine account_id
    let account_id = match (req.account_id, req.account_name) {
        (Some(id), None) => {
            // Verify account belongs to user
            let account_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND user_id = $2)"
            )
            .bind(id)
            .bind(auth.user_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;
            
            if !account_exists {
                return Err(AppError::BadRequest("Account not found or you don't have permission to use it".to_string()));
            }
            id
        }
        (None, Some(name)) => {
            // Create new account
            if name.is_empty() || name.len() > 50 {
                return Err(AppError::BadRequest("Account name must be between 1 and 50 characters".to_string()));
            }
            
            let account_id = sqlx::query_scalar::<_, i32>(
                "INSERT INTO accounts (user_id, name, type, currency) VALUES ($1, $2, $2, 'USD') RETURNING id"
            )
            .bind(auth.user_id)
            .bind(&name)
            .fetch_one(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;
            
            account_id
        }
        _ => return Err(AppError::BadRequest("Either account_id or account_name must be provided".to_string())),
    };

    // Parse amount
    let amount_str = req.amount.trim();
    let amount_decimal: rust_decimal::Decimal = amount_str.parse()
        .map_err(|_| AppError::BadRequest("Invalid amount format".to_string()))?;

    // Determine transaction type based on amount sign
    let transaction_type = if amount_decimal > rust_decimal::Decimal::ZERO {
        "income"
    } else if amount_decimal < rust_decimal::Decimal::ZERO {
        "expense"
    } else {
        return Err(AppError::BadRequest("Amount cannot be zero".to_string()));
    };

    // Parse date
    let date = if let Some(date_str) = req.date {
        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid date format. Use YYYY-MM-DD".to_string()))?
    } else {
        chrono::Utc::now().date_naive()
    };

    // Determine category_id
    let category_id = match (req.category_id, req.category_name) {
        (Some(id), None) => {
            // Verify category belongs to user
            let category_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1 AND user_id = $2)"
            )
            .bind(id)
            .bind(auth.user_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;
            
            if !category_exists {
                return Err(AppError::BadRequest("Category not found or you don't have permission to use it".to_string()));
            }
            Some(id)
        }
        (None, Some(name)) => {
            // Create new category if it doesn't exist
            if name.is_empty() || name.len() > 50 {
                return Err(AppError::BadRequest("Category name must be between 1 and 50 characters".to_string()));
            }
            
            // Check if category already exists
            let existing_id = sqlx::query_scalar::<_, i32>(
                "SELECT id FROM categories WHERE name = $1 AND user_id = $2"
            )
            .bind(&name)
            .bind(auth.user_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;
            
            if let Some(id) = existing_id {
                Some(id)
            } else {
                // Create new category
                let new_id = sqlx::query_scalar::<_, i32>(
                    "INSERT INTO categories (user_id, name) VALUES ($1, $2) RETURNING id"
                )
                .bind(auth.user_id)
                .bind(&name)
                .fetch_one(&pool)
                .await
                .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;
                
                Some(new_id)
            }
        }
        (None, None) => None, // No category for transfers
        (Some(_), Some(_)) => return Err(AppError::BadRequest("Provide either category_id or category_name, not both".to_string())),
    };

    // Start transaction to ensure atomicity
    let mut tx = pool.begin().await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Insert transaction (using amount as string and casting in SQL)
    let transaction_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO transactions (user_id, account_id, category_id, amount, type, date, description) 
         VALUES ($1, $2, $3, $4::numeric, $5, $6, $7) RETURNING id"
    )
    .bind(auth.user_id)
    .bind(account_id)
    .bind(&category_id)
    .bind(&amount_str)
    .bind(transaction_type)
    .bind(date)
    .bind(&req.description)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Update account balance (using amount as string and casting in SQL)
    sqlx::query(
        "UPDATE accounts SET balance = balance + $1::numeric WHERE id = $2"
    )
    .bind(&amount_str)
    .bind(account_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Commit transaction
    tx.commit().await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Fetch complete transaction data for response
    let row = sqlx::query(
        "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                t.amount::text, t.type, t.date, t.description, t.created_at
         FROM transactions t
         JOIN accounts a ON t.account_id = a.id
         LEFT JOIN categories c ON t.category_id = c.id
         WHERE t.id = $1 AND t.user_id = $2"
    )
    .bind(transaction_id)
    .bind(auth.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let transaction = TransactionResponse {
        id: row.get(0),
        account_id: row.get(2),
        account_name: row.get(3),
        category_id: row.get(4),
        category_name: row.get(5),
        amount: row.get(6),
        r#type: row.get(7),
        date: row.get(8),
        description: row.get(9),
        created_at: row.get(10),
    };

    Ok(Json(CreateTransactionResponse {
        message: "Transaction created successfully".to_string(),
        transaction,
    }))
}

pub async fn get_transactions(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Query(params): Query<GetTransactionsQuery>,
) -> Result<Json<Vec<TransactionResponse>>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    // Parse dates if provided
    let start_date = if let Some(start_str) = &params.start_date {
        Some(NaiveDate::parse_from_str(start_str, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid start_date format. Use YYYY-MM-DD".to_string()))?)
    } else {
        None
    };

    let end_date = if let Some(end_str) = &params.end_date {
        Some(NaiveDate::parse_from_str(end_str, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid end_date format. Use YYYY-MM-DD".to_string()))?)
    } else {
        None
    };

    // Build query based on provided filters
    let rows = match (params.account_id, start_date, end_date) {
        (Some(acc_id), Some(start), Some(end)) => {
            sqlx::query(
                "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                        t.amount::text, t.type, t.date, t.description, t.created_at
                 FROM transactions t
                 JOIN accounts a ON t.account_id = a.id
                 LEFT JOIN categories c ON t.category_id = c.id
                 WHERE t.user_id = $1 AND t.account_id = $2 AND t.date >= $3 AND t.date <= $4
                 ORDER BY t.date DESC, t.created_at DESC"
            )
            .bind(auth.user_id)
            .bind(acc_id)
            .bind(start)
            .bind(end)
            .fetch_all(&pool)
            .await
        }
        (Some(acc_id), Some(start), None) => {
            sqlx::query(
                "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                        t.amount::text, t.type, t.date, t.description, t.created_at
                 FROM transactions t
                 JOIN accounts a ON t.account_id = a.id
                 LEFT JOIN categories c ON t.category_id = c.id
                 WHERE t.user_id = $1 AND t.account_id = $2 AND t.date >= $3
                 ORDER BY t.date DESC, t.created_at DESC"
            )
            .bind(auth.user_id)
            .bind(acc_id)
            .bind(start)
            .fetch_all(&pool)
            .await
        }
        (Some(acc_id), None, Some(end)) => {
            sqlx::query(
                "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                        t.amount::text, t.type, t.date, t.description, t.created_at
                 FROM transactions t
                 JOIN accounts a ON t.account_id = a.id
                 LEFT JOIN categories c ON t.category_id = c.id
                 WHERE t.user_id = $1 AND t.account_id = $2 AND t.date <= $3
                 ORDER BY t.date DESC, t.created_at DESC"
            )
            .bind(auth.user_id)
            .bind(acc_id)
            .bind(end)
            .fetch_all(&pool)
            .await
        }
        (Some(acc_id), None, None) => {
            sqlx::query(
                "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                        t.amount::text, t.type, t.date, t.description, t.created_at
                 FROM transactions t
                 JOIN accounts a ON t.account_id = a.id
                 LEFT JOIN categories c ON t.category_id = c.id
                 WHERE t.user_id = $1 AND t.account_id = $2
                 ORDER BY t.date DESC, t.created_at DESC"
            )
            .bind(auth.user_id)
            .bind(acc_id)
            .fetch_all(&pool)
            .await
        }
        (None, Some(start), Some(end)) => {
            sqlx::query(
                "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                        t.amount::text, t.type, t.date, t.description, t.created_at
                 FROM transactions t
                 JOIN accounts a ON t.account_id = a.id
                 LEFT JOIN categories c ON t.category_id = c.id
                 WHERE t.user_id = $1 AND t.date >= $2 AND t.date <= $3
                 ORDER BY t.date DESC, t.created_at DESC"
            )
            .bind(auth.user_id)
            .bind(start)
            .bind(end)
            .fetch_all(&pool)
            .await
        }
        (None, Some(start), None) => {
            sqlx::query(
                "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                        t.amount::text, t.type, t.date, t.description, t.created_at
                 FROM transactions t
                 JOIN accounts a ON t.account_id = a.id
                 LEFT JOIN categories c ON t.category_id = c.id
                 WHERE t.user_id = $1 AND t.date >= $2
                 ORDER BY t.date DESC, t.created_at DESC"
            )
            .bind(auth.user_id)
            .bind(start)
            .fetch_all(&pool)
            .await
        }
        (None, None, Some(end)) => {
            sqlx::query(
                "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                        t.amount::text, t.type, t.date, t.description, t.created_at
                 FROM transactions t
                 JOIN accounts a ON t.account_id = a.id
                 LEFT JOIN categories c ON t.category_id = c.id
                 WHERE t.user_id = $1 AND t.date <= $2
                 ORDER BY t.date DESC, t.created_at DESC"
            )
            .bind(auth.user_id)
            .bind(end)
            .fetch_all(&pool)
            .await
        }
        (None, None, None) => {
            sqlx::query(
                "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                        t.amount::text, t.type, t.date, t.description, t.created_at
                 FROM transactions t
                 JOIN accounts a ON t.account_id = a.id
                 LEFT JOIN categories c ON t.category_id = c.id
                 WHERE t.user_id = $1
                 ORDER BY t.date DESC, t.created_at DESC"
            )
            .bind(auth.user_id)
            .fetch_all(&pool)
            .await
        }
    }
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let transactions: Vec<TransactionResponse> = rows
        .into_iter()
        .map(|row| TransactionResponse {
            id: row.get(0),
            account_id: row.get(2),
            account_name: row.get(3),
            category_id: row.get(4),
            category_name: row.get(5),
            amount: row.get(6),
            r#type: row.get(7),
            date: row.get(8),
            description: row.get(9),
            created_at: row.get(10),
        })
        .collect();

    Ok(Json(transactions))
}

pub async fn transfer(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    // Validate accounts belong to user
    let from_account_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND user_id = $2)"
    )
    .bind(req.from_account_id)
    .bind(auth.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let to_account_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND user_id = $2)"
    )
    .bind(req.to_account_id)
    .bind(auth.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    if !from_account_exists {
        return Err(AppError::BadRequest("From account not found or you don't have permission to use it".to_string()));
    }

    if !to_account_exists {
        return Err(AppError::BadRequest("To account not found or you don't have permission to use it".to_string()));
    }

    if req.from_account_id == req.to_account_id {
        return Err(AppError::BadRequest("Cannot transfer to the same account".to_string()));
    }

    // Parse amount
    let amount_str = req.amount.trim();
    let amount_decimal: rust_decimal::Decimal = amount_str.parse()
        .map_err(|_| AppError::BadRequest("Invalid amount format".to_string()))?;

    if amount_decimal <= rust_decimal::Decimal::ZERO {
        return Err(AppError::BadRequest("Transfer amount must be positive".to_string()));
    }

    // Check if from_account has sufficient balance
    let from_balance: rust_decimal::Decimal = sqlx::query_scalar::<_, String>(
        "SELECT balance::text FROM accounts WHERE id = $1"
    )
    .bind(req.from_account_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
    .parse()
    .map_err(|_| AppError::InternalServerError("Failed to parse balance".to_string()))?;

    if from_balance < amount_decimal {
        return Err(AppError::BadRequest(format!("Insufficient balance. Account has {}, but trying to transfer {}", from_balance, amount_decimal)));
    }

    // Parse date
    let date = if let Some(date_str) = req.date {
        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid date format. Use YYYY-MM-DD".to_string()))?
    } else {
        chrono::Utc::now().date_naive()
    };

    // Build description
    let description = req.description.or_else(|| {
        Some(format!("Transfer from account {} to account {}", req.from_account_id, req.to_account_id))
    });

    // Start transaction to ensure atomicity
    let mut tx = pool.begin().await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Create transaction for from_account (negative amount)
    let from_amount = format!("-{}", amount_str);
    let from_transaction_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO transactions (user_id, account_id, category_id, amount, type, date, description) 
         VALUES ($1, $2, NULL, $3::numeric, 'transfer', $4, $5) RETURNING id"
    )
    .bind(auth.user_id)
    .bind(req.from_account_id)
    .bind(&from_amount)
    .bind(date)
    .bind(&description)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Create transaction for to_account (positive amount)
    let to_transaction_id = sqlx::query_scalar::<_, i32>(
        "INSERT INTO transactions (user_id, account_id, category_id, amount, type, date, description) 
         VALUES ($1, $2, NULL, $3::numeric, 'transfer', $4, $5) RETURNING id"
    )
    .bind(auth.user_id)
    .bind(req.to_account_id)
    .bind(&amount_str)
    .bind(date)
    .bind(&description)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Update from_account balance (subtract)
    sqlx::query(
        "UPDATE accounts SET balance = balance - $1::numeric WHERE id = $2"
    )
    .bind(&amount_str)
    .bind(req.from_account_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Update to_account balance (add)
    sqlx::query(
        "UPDATE accounts SET balance = balance + $1::numeric WHERE id = $2"
    )
    .bind(&amount_str)
    .bind(req.to_account_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Commit transaction
    tx.commit().await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    // Fetch complete transaction data for response
    let from_row = sqlx::query(
        "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                t.amount::text, t.type, t.date, t.description, t.created_at
         FROM transactions t
         JOIN accounts a ON t.account_id = a.id
         LEFT JOIN categories c ON t.category_id = c.id
         WHERE t.id = $1 AND t.user_id = $2"
    )
    .bind(from_transaction_id)
    .bind(auth.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let to_row = sqlx::query(
        "SELECT t.id, t.user_id, t.account_id, a.name as account_name, t.category_id, c.name as category_name, 
                t.amount::text, t.type, t.date, t.description, t.created_at
         FROM transactions t
         JOIN accounts a ON t.account_id = a.id
         LEFT JOIN categories c ON t.category_id = c.id
         WHERE t.id = $1 AND t.user_id = $2"
    )
    .bind(to_transaction_id)
    .bind(auth.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let from_transaction = TransactionResponse {
        id: from_row.get(0),
        account_id: from_row.get(2),
        account_name: from_row.get(3),
        category_id: from_row.get(4),
        category_name: from_row.get(5),
        amount: from_row.get(6),
        r#type: from_row.get(7),
        date: from_row.get(8),
        description: from_row.get(9),
        created_at: from_row.get(10),
    };

    let to_transaction = TransactionResponse {
        id: to_row.get(0),
        account_id: to_row.get(2),
        account_name: to_row.get(3),
        category_id: to_row.get(4),
        category_name: to_row.get(5),
        amount: to_row.get(6),
        r#type: to_row.get(7),
        date: to_row.get(8),
        description: to_row.get(9),
        created_at: to_row.get(10),
    };

    Ok(Json(TransferResponse {
        message: "Transfer completed successfully".to_string(),
        from_transaction,
        to_transaction,
    }))
}

