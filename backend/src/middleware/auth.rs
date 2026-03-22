use axum::http::{header, HeaderMap};
use uuid::Uuid;

use crate::db::Store;
use crate::domain::{Receiver, User};
use crate::error::AppError;

pub async fn require_receiver(
    headers: &HeaderMap,
    store: &dyn Store,
) -> Result<Receiver, AppError> {
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let token = value
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?
        .trim();

    if token.is_empty() {
        return Err(AppError::Unauthorized);
    }

    store
        .get_receiver_by_token(token)
        .await?
        .ok_or(AppError::Unauthorized)
}

pub fn require_user_id(headers: &HeaderMap) -> Result<Uuid, AppError> {
    let value = headers
        .get("x-user-id")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::bad_request("x-user-id header is required"))?;

    Uuid::parse_str(value)
        .map_err(|_| AppError::bad_request("x-user-id must be a valid UUID"))
}

pub async fn require_user(headers: &HeaderMap, store: &dyn Store) -> Result<User, AppError> {
    let user_id = require_user_id(headers)?;
    store
        .get_user_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound("user"))
}
