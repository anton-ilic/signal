pub mod postgres;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{
    Button, ButtonEvent, DeviceInventory, EventFeedItem, NewButton, NewButtonEvent, NewPushToken,
    NewReceiver, PushToken, Receiver, User,
};
use crate::error::AppError;

#[async_trait]
pub trait Store: Send + Sync {
    async fn create_user(&self, email: String) -> Result<User, AppError>;
    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, AppError>;
    async fn create_receiver(&self, receiver: NewReceiver) -> Result<Receiver, AppError>;
    async fn get_receiver_by_id(&self, receiver_id: Uuid) -> Result<Option<Receiver>, AppError>;
    async fn get_receiver_by_token(&self, auth_token: &str) -> Result<Option<Receiver>, AppError>;
    async fn touch_receiver(&self, receiver_id: Uuid) -> Result<Receiver, AppError>;
    async fn create_button(&self, button: NewButton) -> Result<Button, AppError>;
    async fn get_button_by_id(&self, button_id: &str) -> Result<Option<Button>, AppError>;
    async fn list_devices_for_user(&self, user_id: Uuid) -> Result<DeviceInventory, AppError>;
    async fn find_event_by_counter(
        &self,
        button_id: &str,
        event_counter: i64,
    ) -> Result<Option<ButtonEvent>, AppError>;
    async fn insert_button_event(&self, event: NewButtonEvent) -> Result<ButtonEvent, AppError>;
    async fn list_events_for_user(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<EventFeedItem>, AppError>;
    async fn upsert_push_token(&self, token: NewPushToken) -> Result<PushToken, AppError>;
    async fn list_push_tokens_for_user(&self, user_id: Uuid) -> Result<Vec<PushToken>, AppError>;
}
