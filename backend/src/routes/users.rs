use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::domain::CreateUserRequest;
use crate::error::AppError;
use crate::AppState;

pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<crate::domain::User>), AppError> {
    let email = request.email.trim().to_lowercase();
    if email.is_empty() {
        return Err(AppError::bad_request("email is required"));
    }

    let user = state.store.create_user(email).await?;

    Ok((StatusCode::CREATED, Json(user)))
}
