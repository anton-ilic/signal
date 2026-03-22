use axum::Json;

use crate::domain::HealthResponse;

pub async fn root() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "signal-backend",
        version: env!("CARGO_PKG_VERSION"),
    })
}

pub async fn health() -> Json<HealthResponse> {
    root().await
}
