use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub message: String,
    pub token: String,
    pub user_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountResponse {
    pub id: i32,
    pub name: String,
    pub currency: String,
    pub balance: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionResponse {
    pub id: i32,
    pub account_name: String,
    pub category_name: Option<String>,
    pub amount: String,
    pub r#type: String, // 'income', 'expense', 'transfer'
    pub date: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BudgetResponse {
    pub id: i32,
    pub category_name: Option<String>,
    pub amount: String,     // 预算总额
    pub spent: String,      // 已用
    pub remaining: String,  // 剩余
    pub is_over_budget: bool,
    pub period: Option<String>,
}