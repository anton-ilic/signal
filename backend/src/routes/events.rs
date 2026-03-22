use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;

use crate::domain::{ButtonPressRequest, ListEventsQuery};
use crate::error::AppError;
use crate::middleware::auth::{require_receiver, require_user};
use crate::services::events::ingest_button_press as ingest_service;
use crate::AppState;

pub async fn ingest_button_press(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ButtonPressRequest>,
) -> Result<(StatusCode, Json<crate::domain::ButtonPressOutcome>), AppError> {
    let receiver = require_receiver(&headers, state.store.as_ref()).await?;
    let outcome = ingest_service(
        state.store.as_ref(),
        state.notifications.as_ref(),
        &receiver,
        request,
    )
    .await?;

    let status = if outcome.deduplicated {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };

    Ok((status, Json(outcome)))
}

pub async fn list_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListEventsQuery>,
) -> Result<Json<Vec<crate::domain::EventFeedItem>>, AppError> {
    let user = require_user(&headers, state.store.as_ref()).await?;
    let limit = i64::from(query.limit.unwrap_or(50)).clamp(1, 200);
    let events = state.store.list_events_for_user(user.id, limit).await?;

    Ok(Json(events))
}
