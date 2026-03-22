use std::env;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub cors_allow_origin: String,
    pub log_filter: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            host: env::var("SIGNAL_BACKEND_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: read_env_or("SIGNAL_BACKEND_PORT", 8080)?,
            database_url: env::var("DATABASE_URL").map_err(|_| {
                AppError::Config("DATABASE_URL must be set for the backend to start".into())
            })?,
            database_max_connections: read_env_or("DATABASE_MAX_CONNECTIONS", 5)?,
            cors_allow_origin: env::var("CORS_ALLOW_ORIGIN").unwrap_or_else(|_| "*".to_string()),
            log_filter: env::var("RUST_LOG")
                .unwrap_or_else(|_| "signal_backend=debug,tower_http=info".to_string()),
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn read_env_or<T>(name: &str, default: T) -> Result<T, AppError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(name) {
        Ok(value) => value
            .parse::<T>()
            .map_err(|err| AppError::Config(format!("invalid value for {name}: {err}"))),
        Err(_) => Ok(default),
    }
}
