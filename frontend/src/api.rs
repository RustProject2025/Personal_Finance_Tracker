use reqwest::{Client, StatusCode};
use crate::models::{LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, AccountResponse, TransactionResponse, BudgetResponse}; // 引入 Register 相关
use anyhow::{Result, anyhow};

const BASE_URL: &str = "http://localhost:3000/api";

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    pub token: Option<String>,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            token: None,
        }
    }

    // 新增注册方法
    pub async fn register(&self, req: RegisterRequest) -> Result<String> {
        let resp = self.client.post(format!("{}/auth/register", BASE_URL))
            .json(&req)
            .send()
            .await?;

        if resp.status() == StatusCode::OK {
            let data: RegisterResponse = resp.json().await?;
            Ok(data.message)
        } else {
            // 尝试读取错误信息
            let err_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(anyhow!("Registration failed: {}", err_text))
        }
    }

    pub async fn login(&mut self, req: LoginRequest) -> Result<()> {
        let resp = self.client.post(format!("{}/auth/login", BASE_URL))
            .json(&req)
            .send()
            .await?;

        if resp.status() == StatusCode::OK {
            let data: LoginResponse = resp.json().await?;
            self.token = Some(data.token);
            Ok(())
        } else {
            Err(anyhow!("Login failed: Check username/password"))
        }
    }

    async fn get_authenticated<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        if let Some(token) = &self.token {
            let resp = self.client.get(format!("{}{}", BASE_URL, endpoint))
                .bearer_auth(token)
                .send()
                .await?;
            
            if resp.status() == StatusCode::OK {
                let data: T = resp.json().await?;
                Ok(data)
            } else {
                Err(anyhow!("Request failed: {}", resp.status()))
            }
        } else {
            Err(anyhow!("Not authenticated"))
        }
    }

    pub async fn get_accounts(&self) -> Result<Vec<AccountResponse>> {
        self.get_authenticated("/accounts").await
    }

    pub async fn get_transactions(&self) -> Result<Vec<TransactionResponse>> {
        self.get_authenticated("/transactions").await
    }

    pub async fn get_budgets(&self) -> Result<Vec<BudgetResponse>> {
        self.get_authenticated("/budgets").await
    }
}