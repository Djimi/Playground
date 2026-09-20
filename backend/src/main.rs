use anyhow::{Context, Result};
use sofia_playgrounds_backend::{config::Config, db, graphql};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;
    let pool = db::connect_and_migrate(&config.database_url).await?;
    let app = graphql::router(pool, &config.frontend_origin)?;
    let listener = TcpListener::bind(config.api_addr)
        .await
        .with_context(|| format!("bind API to {}", config.api_addr))?;

    info!(address = %config.api_addr, "API listening");
    axum::serve(listener, app).await.context("serve API")
}
