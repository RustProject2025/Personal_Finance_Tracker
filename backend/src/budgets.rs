use axum::{
    extract::{State, Path},
    http::HeaderMap,
    Json,
};
use sqlx::{PgPool, Row};
use chrono::{NaiveDate, Utc};

use crate::models::{
    CreateBudgetRequest, CreateBudgetResponse, BudgetResponse,
    UpdateBudgetRequest, UpdateBudgetResponse,
    DeleteBudgetResponse,
};
use crate::auth::AppError;
use crate::middleware;

pub async fn create_budget(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<CreateBudgetRequest>,
) -> Result<Json<CreateBudgetResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    // Validate amount
    let amount_str = req.amount.trim();
    let _amount_decimal: rust_decimal::Decimal = amount_str.parse()
        .map_err(|_| AppError::BadRequest("Invalid amount format".to_string()))?;

    if amount_str.parse::<f64>().unwrap_or(0.0) <= 0.0 {
        return Err(AppError::BadRequest("Budget amount must be positive".to_string()));
    }

    // If category_id is provided, verify it belongs to user
    if let Some(category_id) = req.category_id {
        let category_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1 AND user_id = $2)"
        )
        .bind(category_id)
        .bind(auth.user_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

        if !category_exists {
            return Err(AppError::BadRequest("Category not found or you don't have permission to use it".to_string()));
        }
    }

    // Parse start_date
    let start_date = if let Some(date_str) = req.start_date {
        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid start_date format. Use YYYY-MM-DD".to_string()))?
    } else {
        Utc::now().date_naive()
    };

    let period = req.period.unwrap_or_else(|| "monthly".to_string());

    // Insert budget
    let row = sqlx::query(
        "INSERT INTO budgets (user_id, category_id, amount, period, start_date) 
         VALUES ($1, $2, $3::numeric, $4, $5) 
         RETURNING id, user_id, category_id, amount::text, period, start_date, created_at, updated_at"
    )
    .bind(auth.user_id)
    .bind(&req.category_id)
    .bind(&amount_str)
    .bind(&period)
    .bind(start_date)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let budget_id: i32 = row.get(0);
    let spent = calculate_spent(&pool, auth.user_id, req.category_id, &start_date, &period).await?;
    let budget_amount: rust_decimal::Decimal = amount_str.parse().unwrap();
    let spent_decimal: rust_decimal::Decimal = spent.parse().unwrap_or(rust_decimal::Decimal::ZERO);
    let remaining = budget_amount - spent_decimal;
    let is_over_budget = spent_decimal > budget_amount;

    let category_name = if let Some(cat_id) = req.category_id {
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT name FROM categories WHERE id = $1"
        )
        .bind(cat_id)
        .fetch_one(&pool)
        .await
        .ok()
        .and_then(|r| r)
    } else {
        None
    };

    let budget = BudgetResponse {
        id: budget_id,
        category_id: req.category_id,
        category_name,
        amount: amount_str.to_string(),
        period: Some(period),
        start_date,
        spent,
        remaining: remaining.to_string(),
        is_over_budget,
        created_at: row.get(6),
        updated_at: row.get(7),
    };

    Ok(Json(CreateBudgetResponse {
        message: "Budget created successfully".to_string(),
        budget,
    }))
}

pub async fn get_budgets(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<BudgetResponse>>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    let rows = sqlx::query(
        "SELECT id, user_id, category_id, amount::text, period, start_date, created_at, updated_at 
         FROM budgets WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(auth.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let mut budgets_response = Vec::new();

    for row in rows {
        let budget_id: i32 = row.get(0);
        let category_id: Option<i32> = row.get(2);
        let amount_str: String = row.get(3);
        let period: Option<String> = row.get(4);
        let start_date: NaiveDate = row.get(5);

        let spent = calculate_spent(&pool, auth.user_id, category_id, &start_date, period.as_deref().unwrap_or("monthly")).await?;

        let category_name = if let Some(cat_id) = category_id {
            sqlx::query_scalar::<_, Option<String>>(
                "SELECT name FROM categories WHERE id = $1"
            )
            .bind(cat_id)
            .fetch_one(&pool)
            .await
            .ok()
            .and_then(|r| r)
        } else {
            None
        };

        let budget_amount: rust_decimal::Decimal = amount_str.parse().unwrap_or(rust_decimal::Decimal::ZERO);
        let spent_decimal: rust_decimal::Decimal = spent.parse().unwrap_or(rust_decimal::Decimal::ZERO);
        let remaining = budget_amount - spent_decimal;
        let is_over_budget = spent_decimal > budget_amount;

        budgets_response.push(BudgetResponse {
            id: budget_id,
            category_id,
            category_name,
            amount: amount_str,
            period,
            start_date,
            spent,
            remaining: remaining.to_string(),
            is_over_budget,
            created_at: row.get(6),
            updated_at: row.get(7),
        });
    }

    Ok(Json(budgets_response))
}

pub async fn update_budget(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(budget_id): Path<i32>,
    Json(req): Json<UpdateBudgetRequest>,
) -> Result<Json<UpdateBudgetResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    // Validate amount
    let amount_str = req.amount.trim();
    let _amount_decimal: rust_decimal::Decimal = amount_str.parse()
        .map_err(|_| AppError::BadRequest("Invalid amount format".to_string()))?;

    if amount_str.parse::<f64>().unwrap_or(0.0) <= 0.0 {
        return Err(AppError::BadRequest("Budget amount must be positive".to_string()));
    }

    let period = req.period.unwrap_or_else(|| "monthly".to_string());
    let start_date = if let Some(date_str) = req.start_date {
        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid start_date format. Use YYYY-MM-DD".to_string()))?
    } else {
        Utc::now().date_naive()
    };

    // Update budget
    let row = sqlx::query(
        "UPDATE budgets SET amount = $1::numeric, period = $2, start_date = $3, updated_at = NOW() 
         WHERE id = $4 AND user_id = $5 
         RETURNING id, user_id, category_id, amount::text, period, start_date, created_at, updated_at"
    )
    .bind(&amount_str)
    .bind(&period)
    .bind(start_date)
    .bind(budget_id)
    .bind(auth.user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let row = match row {
        Some(r) => r,
        None => return Err(AppError::BadRequest("Budget not found or you don't have permission to update it".to_string())),
    };

    let category_id: Option<i32> = row.get(2);
    let spent = calculate_spent(&pool, auth.user_id, category_id, &start_date, &period).await?;

    let category_name = if let Some(cat_id) = category_id {
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT name FROM categories WHERE id = $1"
        )
        .bind(cat_id)
        .fetch_one(&pool)
        .await
        .ok()
        .and_then(|r| r)
    } else {
        None
    };

    let budget_amount: rust_decimal::Decimal = amount_str.parse().unwrap_or(rust_decimal::Decimal::ZERO);
    let spent_decimal: rust_decimal::Decimal = spent.parse().unwrap_or(rust_decimal::Decimal::ZERO);
    let remaining = budget_amount - spent_decimal;
    let is_over_budget = spent_decimal > budget_amount;

    let budget = BudgetResponse {
        id: budget_id,
        category_id,
        category_name,
        amount: amount_str.to_string(),
        period: Some(period),
        start_date,
        spent,
        remaining: remaining.to_string(),
        is_over_budget,
        created_at: row.get(6),
        updated_at: row.get(7),
    };

    Ok(Json(UpdateBudgetResponse {
        message: "Budget updated successfully".to_string(),
        budget,
    }))
}

pub async fn delete_budget(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(budget_id): Path<i32>,
) -> Result<Json<DeleteBudgetResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    let result = sqlx::query(
        "DELETE FROM budgets WHERE id = $1 AND user_id = $2"
    )
    .bind(budget_id)
    .bind(auth.user_id)
    .execute(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(AppError::BadRequest("Budget not found or you don't have permission to delete it".to_string()));
    }

    Ok(Json(DeleteBudgetResponse {
        message: "Budget deleted successfully".to_string(),
    }))
}

async fn calculate_spent(
    pool: &PgPool,
    user_id: i32,
    category_id: Option<i32>,
    start_date: &NaiveDate,
    period: &str,
) -> Result<String, AppError> {
    // Build query based on category_id and period
    let query = if let Some(cat_id) = category_id {
        // Category-specific budget: sum expenses for this category
        if period == "monthly" {
            // For monthly budgets, calculate current month's spending (automatically rolls over each month)
            // This ensures the budget resets every month automatically
            sqlx::query_scalar::<_, Option<String>>(
                "SELECT COALESCE(SUM(ABS(amount))::text, '0') 
                 FROM transactions 
                 WHERE user_id = $1 
                 AND category_id = $2 
                 AND type = 'expense' 
                 AND date >= DATE_TRUNC('month', CURRENT_DATE)
                 AND date < DATE_TRUNC('month', CURRENT_DATE) + INTERVAL '1 month'"
            )
            .bind(user_id)
            .bind(cat_id)
            .fetch_one(pool)
            .await
        } else {
            // For other periods, calculate from start_date onwards
            sqlx::query_scalar::<_, Option<String>>(
                "SELECT COALESCE(SUM(ABS(amount))::text, '0') 
                 FROM transactions 
                 WHERE user_id = $1 
                 AND category_id = $2 
                 AND type = 'expense' 
                 AND date >= $3"
            )
            .bind(user_id)
            .bind(cat_id)
            .bind(*start_date)
            .fetch_one(pool)
            .await
        }
    } else {
        // Monthly budget: sum all expenses
        if period == "monthly" {
            // For monthly budgets, calculate current month's spending (automatically rolls over each month)
            // This ensures the budget resets every month automatically
            sqlx::query_scalar::<_, Option<String>>(
                "SELECT COALESCE(SUM(ABS(amount))::text, '0') 
                 FROM transactions 
                 WHERE user_id = $1 
                 AND type = 'expense' 
                 AND date >= DATE_TRUNC('month', CURRENT_DATE)
                 AND date < DATE_TRUNC('month', CURRENT_DATE) + INTERVAL '1 month'"
            )
            .bind(user_id)
            .fetch_one(pool)
            .await
        } else {
            // For other periods, calculate from start_date onwards
            sqlx::query_scalar::<_, Option<String>>(
                "SELECT COALESCE(SUM(ABS(amount))::text, '0') 
                 FROM transactions 
                 WHERE user_id = $1 
                 AND type = 'expense' 
                 AND date >= $2"
            )
            .bind(user_id)
            .bind(*start_date)
            .fetch_one(pool)
            .await
        }
    };

    let result = query
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(result.unwrap_or_else(|| "0".to_string()))
}

