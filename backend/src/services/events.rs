use chrono::Utc;

use crate::db::Store;
use crate::domain::{ButtonPressOutcome, ButtonPressRequest, NewButtonEvent, Receiver};
use crate::error::AppError;
use crate::services::notifications::NotificationSender;

pub async fn ingest_button_press(
    store: &dyn Store,
    notifications: &dyn NotificationSender,
    receiver: &Receiver,
    request: ButtonPressRequest,
) -> Result<ButtonPressOutcome, AppError> {
    if request.button_id.trim().is_empty() {
        return Err(AppError::bad_request("button_id is required"));
    }

    if request.event_counter < 0 {
        return Err(AppError::bad_request("event_counter must be >= 0"));
    }

    let button = store
        .get_button_by_id(&request.button_id)
        .await?
        .ok_or(AppError::NotFound("button"))?;

    if button.receiver_id != receiver.id {
        return Err(AppError::Forbidden(
            "button is not paired with the authenticated receiver",
        ));
    }

    if let Some(existing) = store
        .find_event_by_counter(&request.button_id, request.event_counter)
        .await?
    {
        return Ok(ButtonPressOutcome {
            event: existing,
            deduplicated: true,
        });
    }

    let received_at = request.received_at.unwrap_or_else(Utc::now);
    let pressed_at = request
        .pressed_at
        .unwrap_or_else(|| received_at.to_owned());

    let event = match store
        .insert_button_event(NewButtonEvent {
            button_id: request.button_id.clone(),
            receiver_id: receiver.id,
            event_counter: request.event_counter,
            pressed_at,
            received_at,
        })
        .await
    {
        Ok(event) => event,
        Err(AppError::Conflict(_)) => {
            let existing = store
                .find_event_by_counter(&request.button_id, request.event_counter)
                .await?
                .ok_or_else(|| AppError::internal("dedupe conflict without existing event"))?;

            return Ok(ButtonPressOutcome {
                event: existing,
                deduplicated: true,
            });
        }
        Err(error) => return Err(error),
    };

    let push_tokens = store.list_push_tokens_for_user(button.user_id).await?;
    notifications
        .send_button_pressed(&event, &button, receiver, &push_tokens)
        .await?;

    Ok(ButtonPressOutcome {
        event,
        deduplicated: false,
    })
}
