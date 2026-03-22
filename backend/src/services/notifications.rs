use async_trait::async_trait;
use tracing::info;

use crate::domain::{Button, ButtonEvent, PushToken, Receiver};
use crate::error::AppError;

#[async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send_button_pressed(
        &self,
        event: &ButtonEvent,
        button: &Button,
        receiver: &Receiver,
        tokens: &[PushToken],
    ) -> Result<(), AppError>;
}

pub struct LoggingNotificationSender;

#[async_trait]
impl NotificationSender for LoggingNotificationSender {
    async fn send_button_pressed(
        &self,
        event: &ButtonEvent,
        button: &Button,
        receiver: &Receiver,
        tokens: &[PushToken],
    ) -> Result<(), AppError> {
        info!(
            event_id = %event.id,
            button_id = %button.id,
            receiver_id = %receiver.id,
            push_token_count = tokens.len(),
            "button press accepted; push delivery is currently a logging stub",
        );

        Ok(())
    }
}
