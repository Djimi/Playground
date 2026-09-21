use std::{env, net::SocketAddr};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_addr: SocketAddr,
    pub frontend_origin: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
        let api_addr = env::var("API_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:3000".into())
            .parse()
            .context("API_ADDR must be a socket address")?;
        let frontend_origin =
            env::var("FRONTEND_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".into());

        Ok(Self {
            database_url,
            api_addr,
            frontend_origin,
        })
    }
}
