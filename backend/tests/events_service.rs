use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use signal_backend::db::Store;
use signal_backend::domain::{
    Button, ButtonEvent, ButtonPressRequest, DeviceInventory, EventFeedItem, NewButton,
    NewButtonEvent, NewPushToken, NewReceiver, PushToken, Receiver, User,
};
use signal_backend::error::AppError;
use signal_backend::services::events::ingest_button_press;
use signal_backend::services::notifications::NotificationSender;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
struct TestStore {
    users: Mutex<HashMap<Uuid, User>>,
    receivers: Mutex<HashMap<Uuid, Receiver>>,
    buttons: Mutex<HashMap<String, Button>>,
    events: Mutex<HashMap<(String, i64), ButtonEvent>>,
    push_tokens: Mutex<Vec<PushToken>>,
}

#[async_trait]
impl Store for TestStore {
    async fn create_user(&self, email: String) -> Result<User, AppError> {
        let user = User {
            id: Uuid::new_v4(),
            email,
            created_at: Utc::now(),
        };
        self.users.lock().await.insert(user.id, user.clone());
        Ok(user)
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, AppError> {
        Ok(self.users.lock().await.get(&user_id).cloned())
    }

    async fn create_receiver(&self, receiver: NewReceiver) -> Result<Receiver, AppError> {
        let created = Receiver {
            id: Uuid::new_v4(),
            user_id: receiver.user_id,
            name: receiver.name,
            auth_token: receiver.auth_token,
            last_seen_at: None,
            created_at: Utc::now(),
        };
        self.receivers
            .lock()
            .await
            .insert(created.id, created.clone());
        Ok(created)
    }

    async fn get_receiver_by_id(&self, receiver_id: Uuid) -> Result<Option<Receiver>, AppError> {
        Ok(self.receivers.lock().await.get(&receiver_id).cloned())
    }

    async fn get_receiver_by_token(&self, auth_token: &str) -> Result<Option<Receiver>, AppError> {
        Ok(self
            .receivers
            .lock()
            .await
            .values()
            .find(|receiver| receiver.auth_token == auth_token)
            .cloned())
    }

    async fn touch_receiver(&self, receiver_id: Uuid) -> Result<Receiver, AppError> {
        let mut receivers = self.receivers.lock().await;
        let receiver = receivers
            .get_mut(&receiver_id)
            .ok_or(AppError::NotFound("receiver"))?;
        receiver.last_seen_at = Some(Utc::now());
        Ok(receiver.clone())
    }

    async fn create_button(&self, button: NewButton) -> Result<Button, AppError> {
        let button = Button {
            id: button.id,
            user_id: button.user_id,
            receiver_id: button.receiver_id,
            label: button.label,
            created_at: Utc::now(),
        };

        self.buttons
            .lock()
            .await
            .insert(button.id.clone(), button.clone());

        Ok(button)
    }

    async fn get_button_by_id(&self, button_id: &str) -> Result<Option<Button>, AppError> {
        Ok(self.buttons.lock().await.get(button_id).cloned())
    }

    async fn list_devices_for_user(&self, user_id: Uuid) -> Result<DeviceInventory, AppError> {
        let buttons = self
            .buttons
            .lock()
            .await
            .values()
            .filter(|button| button.user_id == user_id)
            .cloned()
            .collect();
        let receivers = self
            .receivers
            .lock()
            .await
            .values()
            .filter(|receiver| receiver.user_id == user_id)
            .cloned()
            .collect();

        Ok(DeviceInventory { buttons, receivers })
    }

    async fn find_event_by_counter(
        &self,
        button_id: &str,
        event_counter: i64,
    ) -> Result<Option<ButtonEvent>, AppError> {
        Ok(self
            .events
            .lock()
            .await
            .get(&(button_id.to_string(), event_counter))
            .cloned())
    }

    async fn insert_button_event(&self, event: NewButtonEvent) -> Result<ButtonEvent, AppError> {
        let key = (event.button_id.clone(), event.event_counter);
        let mut events = self.events.lock().await;
        if events.contains_key(&key) {
            return Err(AppError::Conflict("event already exists".into()));
        }

        let event = ButtonEvent {
            id: Uuid::new_v4(),
            button_id: event.button_id,
            receiver_id: event.receiver_id,
            event_counter: event.event_counter,
            pressed_at: event.pressed_at,
            received_at: event.received_at,
            created_at: Utc::now(),
        };
        events.insert(key, event.clone());
        Ok(event)
    }

    async fn list_events_for_user(
        &self,
        _user_id: Uuid,
        _limit: i64,
    ) -> Result<Vec<EventFeedItem>, AppError> {
        Ok(Vec::new())
    }

    async fn upsert_push_token(&self, token: NewPushToken) -> Result<PushToken, AppError> {
        let push_token = PushToken {
            id: Uuid::new_v4(),
            user_id: token.user_id,
            platform: token.platform,
            token: token.token,
            created_at: Utc::now(),
            last_seen_at: Utc::now(),
        };

        self.push_tokens.lock().await.push(push_token.clone());
        Ok(push_token)
    }

    async fn list_push_tokens_for_user(&self, user_id: Uuid) -> Result<Vec<PushToken>, AppError> {
        Ok(self
            .push_tokens
            .lock()
            .await
            .iter()
            .filter(|token| token.user_id == user_id)
            .cloned()
            .collect())
    }
}

#[derive(Default)]
struct TestNotifications {
    sent_events: Mutex<Vec<Uuid>>,
}

#[async_trait]
impl NotificationSender for TestNotifications {
    async fn send_button_pressed(
        &self,
        event: &ButtonEvent,
        _button: &Button,
        _receiver: &Receiver,
        _tokens: &[PushToken],
    ) -> Result<(), AppError> {
        self.sent_events.lock().await.push(event.id);
        Ok(())
    }
}

#[tokio::test]
async fn ingest_button_press_creates_event_and_emits_notification() {
    let store = Arc::new(TestStore::default());
    let notifications = Arc::new(TestNotifications::default());
    let user = store
        .create_user("nurse@example.com".into())
        .await
        .expect("user created");
    let receiver = store
        .create_receiver(NewReceiver {
            user_id: user.id,
            name: "Desk receiver".into(),
            auth_token: "receiver-token".into(),
        })
        .await
        .expect("receiver created");
    store
        .create_button(NewButton {
            id: "button-1".into(),
            user_id: user.id,
            receiver_id: receiver.id,
            label: "Room 101".into(),
        })
        .await
        .expect("button created");
    store
        .upsert_push_token(NewPushToken {
            user_id: user.id,
            platform: "ios".into(),
            token: "push-token".into(),
        })
        .await
        .expect("push token created");

    let outcome = ingest_button_press(
        store.as_ref(),
        notifications.as_ref(),
        &receiver,
        ButtonPressRequest {
            button_id: "button-1".into(),
            event_counter: 42,
            pressed_at: None,
            received_at: Some(Utc::now()),
        },
    )
    .await
    .expect("event ingestion succeeded");

    assert!(!outcome.deduplicated);
    assert_eq!(outcome.event.button_id, "button-1");
    assert_eq!(outcome.event.event_counter, 42);
    assert_eq!(notifications.sent_events.lock().await.len(), 1);
}

#[tokio::test]
async fn ingest_button_press_deduplicates_same_counter() {
    let store = Arc::new(TestStore::default());
    let notifications = Arc::new(TestNotifications::default());
    let user = store
        .create_user("triage@example.com".into())
        .await
        .expect("user created");
    let receiver = store
        .create_receiver(NewReceiver {
            user_id: user.id,
            name: "Hall receiver".into(),
            auth_token: "receiver-token".into(),
        })
        .await
        .expect("receiver created");
    store
        .create_button(NewButton {
            id: "button-2".into(),
            user_id: user.id,
            receiver_id: receiver.id,
            label: "Room 102".into(),
        })
        .await
        .expect("button created");

    let first = ingest_button_press(
        store.as_ref(),
        notifications.as_ref(),
        &receiver,
        ButtonPressRequest {
            button_id: "button-2".into(),
            event_counter: 7,
            pressed_at: Some(Utc::now()),
            received_at: Some(Utc::now()),
        },
    )
    .await
    .expect("first event succeeds");

    let second = ingest_button_press(
        store.as_ref(),
        notifications.as_ref(),
        &receiver,
        ButtonPressRequest {
            button_id: "button-2".into(),
            event_counter: 7,
            pressed_at: Some(Utc::now()),
            received_at: Some(Utc::now()),
        },
    )
    .await
    .expect("duplicate event is deduplicated");

    assert!(!first.deduplicated);
    assert!(second.deduplicated);
    assert_eq!(first.event.id, second.event.id);
    assert_eq!(notifications.sent_events.lock().await.len(), 1);
}
