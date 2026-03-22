use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::domain::{
    CreateButtonRequest, CreateReceiverRequest, NewButton, NewReceiver, ReceiverProvisioning,
};
use crate::error::AppError;
use crate::middleware::auth::{require_receiver, require_user};
use crate::AppState;

pub async fn list_devices(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<crate::domain::DeviceInventory>, AppError> {
    let user = require_user(&headers, state.store.as_ref()).await?;
    let inventory = state.store.list_devices_for_user(user.id).await?;

    Ok(Json(inventory))
}

pub async fn create_receiver(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateReceiverRequest>,
) -> Result<(StatusCode, Json<ReceiverProvisioning>), AppError> {
    let user = require_user(&headers, state.store.as_ref()).await?;
    let name = request.name.trim().to_string();

    if name.is_empty() {
        return Err(AppError::bad_request("name is required"));
    }

    let auth_token = generate_auth_token();
    let receiver = state
        .store
        .create_receiver(NewReceiver {
            user_id: user.id,
            name,
            auth_token: auth_token.clone(),
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ReceiverProvisioning {
            receiver,
            auth_token,
        }),
    ))
}

pub async fn create_button(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateButtonRequest>,
) -> Result<(StatusCode, Json<crate::domain::Button>), AppError> {
    let user = require_user(&headers, state.store.as_ref()).await?;
    let button_id = request.id.trim().to_string();
    let label = request.label.trim().to_string();

    if button_id.is_empty() {
        return Err(AppError::bad_request("id is required"));
    }

    if label.is_empty() {
        return Err(AppError::bad_request("label is required"));
    }

    let receiver = state
        .store
        .get_receiver_by_id(request.receiver_id)
        .await?
        .ok_or(AppError::NotFound("receiver"))?;

    if receiver.user_id != user.id {
        return Err(AppError::Forbidden("receiver does not belong to the authenticated user"));
    }

    let button = state
        .store
        .create_button(NewButton {
            id: button_id,
            user_id: user.id,
            receiver_id: request.receiver_id,
            label,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(button)))
}

pub async fn heartbeat(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<crate::domain::Receiver>, AppError> {
    let receiver = require_receiver(&headers, state.store.as_ref()).await?;
    let receiver = state.store.touch_receiver(receiver.id).await?;

    Ok(Json(receiver))
}

fn generate_auth_token() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(48)
        .map(char::from)
        .collect()
}
