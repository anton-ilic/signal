use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::Store;
use crate::domain::{
    Button, ButtonEvent, DeviceInventory, EventFeedItem, NewButton, NewButtonEvent, NewPushToken,
    NewReceiver, PushToken, Receiver, User,
};
use crate::error::AppError;

#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Store for PostgresStore {
    async fn create_user(&self, email: String) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email)
            VALUES ($1)
            ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email
            RETURNING id, email, created_at
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn create_receiver(&self, receiver: NewReceiver) -> Result<Receiver, AppError> {
        let created = sqlx::query_as::<_, Receiver>(
            r#"
            INSERT INTO receivers (user_id, name, auth_token)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, name, auth_token, last_seen_at, created_at
            "#,
        )
        .bind(receiver.user_id)
        .bind(receiver.name)
        .bind(receiver.auth_token)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn get_receiver_by_id(&self, receiver_id: Uuid) -> Result<Option<Receiver>, AppError> {
        let receiver = sqlx::query_as::<_, Receiver>(
            r#"
            SELECT id, user_id, name, auth_token, last_seen_at, created_at
            FROM receivers
            WHERE id = $1
            "#,
        )
        .bind(receiver_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(receiver)
    }

    async fn get_receiver_by_token(&self, auth_token: &str) -> Result<Option<Receiver>, AppError> {
        let receiver = sqlx::query_as::<_, Receiver>(
            r#"
            SELECT id, user_id, name, auth_token, last_seen_at, created_at
            FROM receivers
            WHERE auth_token = $1
            "#,
        )
        .bind(auth_token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(receiver)
    }

    async fn touch_receiver(&self, receiver_id: Uuid) -> Result<Receiver, AppError> {
        let receiver = sqlx::query_as::<_, Receiver>(
            r#"
            UPDATE receivers
            SET last_seen_at = now()
            WHERE id = $1
            RETURNING id, user_id, name, auth_token, last_seen_at, created_at
            "#,
        )
        .bind(receiver_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound("receiver"))?;

        Ok(receiver)
    }

    async fn create_button(&self, button: NewButton) -> Result<Button, AppError> {
        let created = sqlx::query_as::<_, Button>(
            r#"
            INSERT INTO buttons (id, user_id, receiver_id, label)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, receiver_id, label, created_at
            "#,
        )
        .bind(button.id)
        .bind(button.user_id)
        .bind(button.receiver_id)
        .bind(button.label)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| map_unique_violation(err, "button id already exists"))?;

        Ok(created)
    }

    async fn get_button_by_id(&self, button_id: &str) -> Result<Option<Button>, AppError> {
        let button = sqlx::query_as::<_, Button>(
            r#"
            SELECT id, user_id, receiver_id, label, created_at
            FROM buttons
            WHERE id = $1
            "#,
        )
        .bind(button_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(button)
    }

    async fn list_devices_for_user(&self, user_id: Uuid) -> Result<DeviceInventory, AppError> {
        let buttons = sqlx::query_as::<_, Button>(
            r#"
            SELECT id, user_id, receiver_id, label, created_at
            FROM buttons
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let receivers = sqlx::query_as::<_, Receiver>(
            r#"
            SELECT id, user_id, name, auth_token, last_seen_at, created_at
            FROM receivers
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(DeviceInventory { buttons, receivers })
    }

    async fn find_event_by_counter(
        &self,
        button_id: &str,
        event_counter: i64,
    ) -> Result<Option<ButtonEvent>, AppError> {
        let event = sqlx::query_as::<_, ButtonEvent>(
            r#"
            SELECT id, button_id, receiver_id, event_counter, pressed_at, received_at, created_at
            FROM button_events
            WHERE button_id = $1 AND event_counter = $2
            "#,
        )
        .bind(button_id)
        .bind(event_counter)
        .fetch_optional(&self.pool)
        .await?;

        Ok(event)
    }

    async fn insert_button_event(&self, event: NewButtonEvent) -> Result<ButtonEvent, AppError> {
        let created = sqlx::query_as::<_, ButtonEvent>(
            r#"
            INSERT INTO button_events (button_id, receiver_id, event_counter, pressed_at, received_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, button_id, receiver_id, event_counter, pressed_at, received_at, created_at
            "#,
        )
        .bind(event.button_id)
        .bind(event.receiver_id)
        .bind(event.event_counter)
        .bind(event.pressed_at)
        .bind(event.received_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| map_unique_violation(err, "event already exists"))?;

        Ok(created)
    }

    async fn list_events_for_user(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<EventFeedItem>, AppError> {
        let events = sqlx::query_as::<_, EventFeedItem>(
            r#"
            SELECT
                e.id,
                e.button_id,
                b.label AS button_label,
                e.receiver_id,
                r.name AS receiver_name,
                e.event_counter,
                e.pressed_at,
                e.received_at,
                e.created_at
            FROM button_events e
            INNER JOIN buttons b ON b.id = e.button_id
            INNER JOIN receivers r ON r.id = e.receiver_id
            WHERE b.user_id = $1
            ORDER BY e.created_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    async fn upsert_push_token(&self, token: NewPushToken) -> Result<PushToken, AppError> {
        let push_token = sqlx::query_as::<_, PushToken>(
            r#"
            INSERT INTO push_tokens (user_id, platform, token)
            VALUES ($1, $2, $3)
            ON CONFLICT (token) DO UPDATE
            SET user_id = EXCLUDED.user_id,
                platform = EXCLUDED.platform,
                last_seen_at = now()
            RETURNING id, user_id, platform, token, created_at, last_seen_at
            "#,
        )
        .bind(token.user_id)
        .bind(token.platform)
        .bind(token.token)
        .fetch_one(&self.pool)
        .await?;

        Ok(push_token)
    }

    async fn list_push_tokens_for_user(&self, user_id: Uuid) -> Result<Vec<PushToken>, AppError> {
        let tokens = sqlx::query_as::<_, PushToken>(
            r#"
            SELECT id, user_id, platform, token, created_at, last_seen_at
            FROM push_tokens
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tokens)
    }
}

fn map_unique_violation(error: sqlx::Error, message: &'static str) -> AppError {
    match &error {
        sqlx::Error::Database(database_error)
            if database_error.code().as_deref() == Some("23505") =>
        {
            AppError::Conflict(message.to_string())
        }
        _ => AppError::Database(error),
    }
}
