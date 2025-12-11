use reqwest::{Client, StatusCode};
use crate::models::{LoginRequest, LoginResponse, AccountResponse};
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
            Err(anyhow!("Login failed"))
        }
    }

    pub async fn get_accounts(&self) -> Result<Vec<AccountResponse>> {
        if let Some(token) = &self.token {
            let resp = self.client.get(format!("{}/accounts", BASE_URL))
                .bearer_auth(token)
                .send()
                .await?;
            
            if resp.status() == StatusCode::OK {
                let accounts: Vec<AccountResponse> = resp.json().await?;
                Ok(accounts)
            } else {
                Err(anyhow!("Failed to fetch accounts"))
            }
        } else {
            Err(anyhow!("Not authenticated"))
        }
    }
}