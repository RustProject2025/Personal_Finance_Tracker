use reqwest::{Client, StatusCode};
use crate::models::*;
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

  
    pub async fn register(&self, req: RegisterRequest) -> Result<String> {
        let resp = self.client.post(format!("{}/auth/register", BASE_URL))
            .json(&req)
            .send()
            .await?;

        if resp.status() == StatusCode::OK {
            let data: RegisterResponse = resp.json().await?;
            Ok(data.message)
        } else {
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

 
    async fn get_auth<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        if let Some(token) = &self.token {
            let resp = self.client.get(format!("{}{}", BASE_URL, endpoint))
                .bearer_auth(token)
                .send()
                .await?;
            
            if resp.status().is_success() {
                let data: T = resp.json().await?;
                Ok(data)
            } else {
                Err(anyhow!("Request failed: {}", resp.status()))
            }
        } else {
            Err(anyhow!("Not authenticated"))
        }
    }

    async fn post_auth<T: serde::Serialize>(&self, endpoint: &str, body: &T) -> Result<()> {
        if let Some(token) = &self.token {
            let resp = self.client.post(format!("{}{}", BASE_URL, endpoint))
                .bearer_auth(token)
                .json(body)
                .send()
                .await?;
            
            if resp.status().is_success() {
                Ok(())
            } else {
                let err_text = resp.text().await.unwrap_or_default();
                Err(anyhow!("Action failed: {}", err_text))
            }
        } else {
            Err(anyhow!("Not authenticated"))
        }
    }

    async fn delete_auth(&self, endpoint: &str) -> Result<()> {
        if let Some(token) = &self.token {
            let resp = self.client.delete(format!("{}{}", BASE_URL, endpoint))
                .bearer_auth(token)
                .send()
                .await?;
            
            if resp.status().is_success() {
                Ok(())
            } else {
                let err_text = resp.text().await.unwrap_or_default();
                Err(anyhow!("Delete failed: {}", err_text))
            }
        } else {
            Err(anyhow!("Not authenticated"))
        }
    }

    
    pub async fn get_accounts(&self) -> Result<Vec<AccountResponse>> {
        self.get_auth("/accounts").await
    }

    pub async fn get_transactions(&self, account_id: Option<i32>) -> Result<Vec<TransactionResponse>> {
        let url = if let Some(id) = account_id {
            format!("/transactions?account_id={}", id)
        } else {
            "/transactions".to_string()
        };
        self.get_auth(&url).await
    }

    pub async fn get_budgets(&self) -> Result<Vec<BudgetResponse>> {
        self.get_auth("/budgets").await
    }
    
    pub async fn get_categories(&self) -> Result<Vec<CategoryResponse>> {
        self.get_auth("/categories").await
    }

   
    pub async fn create_account(&self, req: CreateAccountRequest) -> Result<()> {
        self.post_auth("/accounts", &req).await
    }

    pub async fn create_transaction(&self, req: CreateTransactionRequest) -> Result<()> {
        self.post_auth("/transactions", &req).await
    }

    pub async fn transfer(&self, req: TransferRequest) -> Result<()> {
        self.post_auth("/transactions/transfer", &req).await
    }

    pub async fn create_category(&self, req: CreateCategoryRequest) -> Result<()> {
        self.post_auth("/categories", &req).await
    }

    pub async fn create_budget(&self, req: CreateBudgetRequest) -> Result<()> {
        self.post_auth("/budgets", &req).await
    }

    
    pub async fn delete_account(&self, id: i32) -> Result<()> {
        self.delete_auth(&format!("/accounts/{}", id)).await
    }

  
    pub async fn delete_category(&self, id: i32) -> Result<()> {
        self.delete_auth(&format!("/categories/{}", id)).await
    }

   
    pub async fn delete_budget(&self, id: i32) -> Result<()> {
        self.delete_auth(&format!("/budgets/{}", id)).await
    }
}