use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use sqlx::{PgPool, Row};

use crate::models::{
    CreateCategoryRequest, CreateCategoryResponse, CategoryResponse,
    DeleteCategoryResponse, Category,
};
use crate::auth::AppError;
use crate::middleware;

pub async fn create_category(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<CreateCategoryRequest>,
) -> Result<Json<CreateCategoryResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    if req.name.is_empty() || req.name.len() > 50 {
        return Err(AppError::BadRequest("Category name must be between 1 and 50 characters".to_string()));
    }

    // If parent_id is provided, verify it belongs to the user
    if let Some(parent_id) = req.parent_id {
        let parent_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE id = $1 AND user_id = $2)"
        )
        .bind(parent_id)
        .bind(auth.user_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

        if !parent_exists {
            return Err(AppError::BadRequest("Parent category not found or you don't have permission to use it".to_string()));
        }
    }

    let row = sqlx::query(
        "INSERT INTO categories (user_id, name, parent_id) VALUES ($1, $2, $3) RETURNING id, user_id, name, parent_id, created_at"
    )
    .bind(auth.user_id)
    .bind(&req.name)
    .bind(&req.parent_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let category = Category {
        id: row.get(0),
        user_id: row.get(1),
        name: row.get(2),
        parent_id: row.get(3),
        created_at: row.get(4),
    };

    Ok(Json(CreateCategoryResponse {
        message: "Category created successfully".to_string(),
        category: category_to_response(category),
    }))
}

pub async fn get_categories(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    let rows = sqlx::query(
        "SELECT id, user_id, name, parent_id, created_at FROM categories WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind(auth.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let categories: Vec<Category> = rows
        .into_iter()
        .map(|row| Category {
            id: row.get(0),
            user_id: row.get(1),
            name: row.get(2),
            parent_id: row.get(3),
            created_at: row.get(4),
        })
        .collect();

    let categories_response: Vec<CategoryResponse> = categories
        .into_iter()
        .map(category_to_response)
        .collect();

    Ok(Json(categories_response))
}

pub async fn delete_category(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    axum::extract::Path(category_id): axum::extract::Path<i32>,
) -> Result<Json<DeleteCategoryResponse>, AppError> {
    let auth = middleware::verify_auth(&pool, &headers).await?;

    // Check if category has child categories
    let has_children = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM categories WHERE parent_id = $1)"
    )
    .bind(category_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    if has_children {
        return Err(AppError::BadRequest("Cannot delete category with child categories".to_string()));
    }

    let result = sqlx::query(
        "DELETE FROM categories WHERE id = $1 AND user_id = $2"
    )
    .bind(category_id)
    .bind(auth.user_id)
    .execute(&pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(AppError::BadRequest("Category not found or you don't have permission to delete it".to_string()));
    }

    Ok(Json(DeleteCategoryResponse {
        message: "Category deleted successfully".to_string(),
    }))
}

fn category_to_response(category: Category) -> CategoryResponse {
    CategoryResponse {
        id: category.id,
        name: category.name,
        parent_id: category.parent_id,
        created_at: category.created_at,
    }
}



