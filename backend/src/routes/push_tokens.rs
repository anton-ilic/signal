use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;

use crate::domain::{NewPushToken, RegisterPushTokenRequest};
use crate::error::AppError;
use crate::middleware::auth::require_user;
use crate::AppState;

pub async fn register_push_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<RegisterPushTokenRequest>,
) -> Result<(StatusCode, Json<crate::domain::PushToken>), AppError> {
    let user = require_user(&headers, state.store.as_ref()).await?;
    let platform = request.platform.trim().to_lowercase();
    let token = request.token.trim().to_string();

    if !matches!(platform.as_str(), "ios" | "android") {
        return Err(AppError::bad_request("platform must be either ios or android"));
    }

    if token.is_empty() {
        return Err(AppError::bad_request("token is required"));
    }

    let push_token = state
        .store
        .upsert_push_token(NewPushToken {
            user_id: user.id,
            platform,
            token,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(push_token)))
}
