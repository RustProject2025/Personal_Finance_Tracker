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