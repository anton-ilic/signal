use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Receiver {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    #[serde(skip_serializing)]
    pub auth_token: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Button {
    pub id: String,
    pub user_id: Uuid,
    pub receiver_id: Uuid,
    pub label: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ButtonEvent {
    pub id: Uuid,
    pub button_id: String,
    pub receiver_id: Uuid,
    pub event_counter: i64,
    pub pressed_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PushToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String,
    #[serde(skip_serializing)]
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventFeedItem {
    pub id: Uuid,
    pub button_id: String,
    pub button_label: String,
    pub receiver_id: Uuid,
    pub receiver_name: String,
    pub event_counter: i64,
    pub pressed_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceInventory {
    pub buttons: Vec<Button>,
    pub receivers: Vec<Receiver>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReceiverProvisioning {
    pub receiver: Receiver,
    pub auth_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ButtonPressOutcome {
    pub event: ButtonEvent,
    pub deduplicated: bool,
}

#[derive(Debug, Clone)]
pub struct NewReceiver {
    pub user_id: Uuid,
    pub name: String,
    pub auth_token: String,
}

#[derive(Debug, Clone)]
pub struct NewButton {
    pub id: String,
    pub user_id: Uuid,
    pub receiver_id: Uuid,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct NewButtonEvent {
    pub button_id: String,
    pub receiver_id: Uuid,
    pub event_counter: i64,
    pub pressed_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewPushToken {
    pub user_id: Uuid,
    pub platform: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateReceiverRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateButtonRequest {
    pub id: String,
    pub receiver_id: Uuid,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterPushTokenRequest {
    pub platform: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct ButtonPressRequest {
    pub button_id: String,
    pub event_counter: i64,
    pub pressed_at: Option<DateTime<Utc>>,
    pub received_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}
