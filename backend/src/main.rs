mod auth;
mod models;
mod middleware;
mod accounts;
mod categories;
mod transactions;
mod budgets;

use axum::{Router, routing::{get, post, put, delete}, Json};
use sqlx::postgres::PgPoolOptions;
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

async fn health_check() -> Json<Health> {
    Json(Health { status: "ok" })
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    println!("Connected to PostgreSQL");

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/accounts", get(accounts::get_accounts))
        .route("/api/accounts", post(accounts::create_account))
        .route("/api/accounts/{id}", put(accounts::update_account))
        .route("/api/accounts/{id}", delete(accounts::delete_account))
        .route("/api/categories", get(categories::get_categories))
        .route("/api/categories", post(categories::create_category))
        .route("/api/categories/{id}", delete(categories::delete_category))
        .route("/api/transactions", get(transactions::get_transactions))
        .route("/api/transactions", post(transactions::create_transaction))
        .route("/api/transactions/transfer", post(transactions::transfer))
        .route("/api/budgets", get(budgets::get_budgets))
        .route("/api/budgets", post(budgets::create_budget))
        .route("/api/budgets/{id}", put(budgets::update_budget))
        .route("/api/budgets/{id}", delete(budgets::delete_budget))
        .layer(CorsLayer::permissive())
        .with_state(pool);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let listener = TcpListener::bind(addr).await?;
    println!("Server running at http://{addr}");

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
