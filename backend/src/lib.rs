pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod services;

use std::sync::Arc;

use axum::Router;

use crate::config::Config;
use crate::db::Store;
use crate::services::notifications::NotificationSender;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub store: Arc<dyn Store>,
    pub notifications: Arc<dyn NotificationSender>,
}

impl AppState {
    pub fn new(
        config: Config,
        store: Arc<dyn Store>,
        notifications: Arc<dyn NotificationSender>,
    ) -> Self {
        Self {
            config,
            store,
            notifications,
        }
    }
}

pub fn app_router(state: AppState) -> Router {
    routes::router(state)
}
