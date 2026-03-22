use std::sync::Arc;

use signal_backend::config::Config;
use signal_backend::db::postgres::PostgresStore;
use signal_backend::services::notifications::LoggingNotificationSender;
use signal_backend::{app_router, AppState};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(config.log_filter.clone())),
        )
        .with_target(false)
        .compact()
        .init();

    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let store = Arc::new(PostgresStore::new(pool));
    let notifications = Arc::new(LoggingNotificationSender);
    let state = AppState::new(config.clone(), store, notifications);
    let app = app_router(state);

    let listener = TcpListener::bind(config.bind_address()).await?;
    info!("signal-backend listening on {}", config.bind_address());

    axum::serve(listener, app).await?;

    Ok(())
}
