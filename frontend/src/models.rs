use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

// 复用 LoginRequest 作为注册请求，因为字段一样
pub type RegisterRequest = LoginRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub message: String,
    pub token: String,
    pub user_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterResponse {
    pub message: String,
    pub user_id: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountResponse {
    pub id: i32,
    pub name: String,
    pub currency: String,
    pub balance: String,
}

// ... TransactionResponse, BudgetResponse 保持不变 ...
// (为了节省篇幅，这里省略 Transaction 和 Budget 的定义，保持你之前的代码即可)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionResponse {
    pub id: i32,
    pub account_name: String,
    pub category_name: Option<String>,
    pub amount: String,
    pub r#type: String, 
    pub date: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BudgetResponse {
    pub id: i32,
    pub category_name: Option<String>,
    pub amount: String,     
    pub spent: String,      
    pub remaining: String,  
    pub is_over_budget: bool,
    pub period: Option<String>,
}