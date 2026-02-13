use crate::domain::Message;
use anyhow::Result;
use sqlx::{MySqlPool, Row};

#[derive(Clone)]
pub struct MessageRepository {
    pool: MySqlPool,
}

impl MessageRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_message(&self, message: &Message) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO chat_messages (id, room_id, sender_id, content, message_type, metadata, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&message.id)
        .bind(&message.room_id)
        .bind(message.sender_id)
        .bind(&message.content)
        .bind(&message.message_type)
        .bind(&message.metadata)
        .bind(message.created_at)
        .bind(message.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_room_messages(
        &self,
        room_id: &str,
        limit: i64,
        before_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        let messages = if let Some(before_id) = before_id {
            sqlx::query(
                r#"
                SELECT id, room_id, sender_id, content, message_type, metadata, created_at, updated_at
                FROM chat_messages
                WHERE room_id = ? AND id < ?
                ORDER BY created_at DESC
                LIMIT ?
                "#
            )
            .bind(room_id)
            .bind(before_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, room_id, sender_id, content, message_type, metadata, created_at, updated_at
                FROM chat_messages
                WHERE room_id = ?
                ORDER BY created_at DESC
                LIMIT ?
                "#
            )
            .bind(room_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        let messages: Vec<Message> = messages
            .iter()
            .map(|row| Message {
                id: row.get("id"),
                room_id: row.get("room_id"),
                sender_id: row.get("sender_id"),
                content: row.get("content"),
                message_type: row.get("message_type"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(messages)
    }

    pub async fn get_message(&self, message_id: &str) -> Result<Option<Message>> {
        let row = sqlx::query(
            r#"
            SELECT id, room_id, sender_id, content, message_type, metadata, created_at, updated_at
            FROM chat_messages
            WHERE id = ?
            "#
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Message {
            id: row.get("id"),
            room_id: row.get("room_id"),
            sender_id: row.get("sender_id"),
            content: row.get("content"),
            message_type: row.get("message_type"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }
}
