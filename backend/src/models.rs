use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub message: String,
    pub user_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub message: String,
    pub token: String,
    pub user_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i32,
    #[allow(dead_code)]
    pub username: String,
    pub password_hash: String,
    #[allow(dead_code)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub is_valid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResponse {
    pub id: i32,
    pub name: String,
    pub currency: String,
    pub balance: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAccountResponse {
    pub message: String,
    pub account: AccountResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAccountResponse {
    pub message: String,
    pub account: AccountResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteAccountResponse {
    pub message: String,
}

#[derive(Debug)]
pub struct Account {
    pub id: i32,
    #[allow(dead_code)]
    pub user_id: i32,
    pub name: String,
    pub currency: String,
    pub balance: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub parent_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryResponse {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoryResponse {
    pub message: String,
    pub category: CategoryResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteCategoryResponse {
    pub message: String,
}

#[derive(Debug, FromRow)]
pub struct Category {
    pub id: i32,
    #[allow(dead_code)]
    pub user_id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub account_id: Option<i32>,
    pub account_name: Option<String>,
    pub amount: String,
    pub date: Option<String>,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub id: i32,
    pub account_id: i32,
    pub account_name: String,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub amount: String,
    pub r#type: String,
    pub date: chrono::NaiveDate,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionResponse {
    pub message: String,
    pub transaction: TransactionResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferRequest {
    pub from_account_id: i32,
    pub to_account_id: i32,
    pub amount: String,
    pub date: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferResponse {
    pub message: String,
    pub from_transaction: TransactionResponse,
    pub to_transaction: TransactionResponse,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Transaction {
    pub id: i32,
    pub user_id: i32,
    pub account_id: i32,
    pub category_id: Option<i32>,
    pub amount: String,
    pub r#type: String,
    pub date: chrono::NaiveDate,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBudgetRequest {
    pub category_id: Option<i32>,
    pub amount: String,
    pub period: Option<String>,
    pub start_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetResponse {
    pub id: i32,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub amount: String,
    pub period: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub spent: String,
    pub remaining: String,
    pub is_over_budget: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBudgetResponse {
    pub message: String,
    pub budget: BudgetResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBudgetRequest {
    pub amount: String,
    pub period: Option<String>,
    pub start_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBudgetResponse {
    pub message: String,
    pub budget: BudgetResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteBudgetResponse {
    pub message: String,
}

#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct Budget {
    pub id: i32,
    pub user_id: i32,
    pub category_id: Option<i32>,
    pub amount: String,
    pub period: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

