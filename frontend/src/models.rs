use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateAccountRequest {
    pub name: String,
    pub currency: Option<String>,
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
pub struct CreateTransactionRequest {
    pub account_id: Option<i32>,   
    pub account_name: Option<String>, 
    pub category_id: Option<i32>,  
    pub amount: String,
    pub r#type: String,
    pub date: String,   
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransferRequest {
    pub from_account_id: i32,
    pub to_account_id: i32,
    pub amount: String,
    pub date: Option<String>,
    pub description: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryResponse {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub parent_id: Option<i32>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateBudgetRequest {
    pub category_id: Option<i32>,
    pub amount: String,
    pub period: Option<String>,
    pub start_date: Option<String>,
}