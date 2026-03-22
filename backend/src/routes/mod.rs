mod devices;
mod events;
mod health;
mod push_tokens;
mod users;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use http::header::HeaderValue;
use http::Method;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::AppState;

pub fn router(state: AppState) -> Router {
    let cors = if state.config.cors_allow_origin == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any)
    } else {
        let origin: HeaderValue = state
            .config
            .cors_allow_origin
            .parse()
            .expect("valid cors origin");

        CorsLayer::new()
            .allow_origin(origin)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any)
    };

    Router::new()
        .route("/", get(health::root))
        .route("/health", get(health::health))
        .route("/v1/users", post(users::create_user))
        .route("/v1/devices", get(devices::list_devices))
        .route("/v1/devices/buttons", post(devices::create_button))
        .route("/v1/devices/receivers", post(devices::create_receiver))
        .route("/v1/receivers/heartbeat", post(devices::heartbeat))
        .route("/v1/events", get(events::list_events))
        .route("/v1/events/button-press", post(events::ingest_button_press))
        .route("/v1/push-tokens", post(push_tokens::register_push_token))
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
