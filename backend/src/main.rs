use axum::{Router, routing::get, Json};
use sqlx::postgres::PgPoolOptions;
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;

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
        .with_state(pool);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let listener = TcpListener::bind(addr).await?;
    println!("Server running at http://{addr}");

    axum::serve(listener, app).await.unwrap();
    Ok(())
}
