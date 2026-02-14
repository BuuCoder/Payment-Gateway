use crate::domain::{RoomInvitation, InvitationResponse};
use anyhow::Result;
use sqlx::{MySqlPool, Row};

#[derive(Clone)]
pub struct InvitationRepository {
    pub pool: MySqlPool,
}

impl InvitationRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_invitation(
        &self,
        room_id: &str,
        user_id: i64,
        invited_by: i64,
    ) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO room_invitations (room_id, user_id, invited_by, status, created_at, updated_at)
            VALUES (?, ?, ?, 'pending', NOW(), NOW())
            "#
        )
        .bind(room_id)
        .bind(user_id)
        .bind(invited_by)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_id() as i64)
    }

    pub async fn get_invitation(&self, invitation_id: i64) -> Result<Option<RoomInvitation>> {
        let row = sqlx::query_as::<_, RoomInvitation>(
            r#"
            SELECT id, room_id, user_id, invited_by, status, created_at, updated_at
            FROM room_invitations
            WHERE id = ?
            "#
        )
        .bind(invitation_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_user_invitations(
        &self,
        user_id: i64,
        status: Option<&str>,
    ) -> Result<Vec<InvitationResponse>> {
        let query = if let Some(status) = status {
            sqlx::query(
                r#"
                SELECT 
                    i.id, i.room_id, i.invited_by, i.status, i.created_at,
                    r.name as room_name, r.room_type,
                    u.name as invited_by_name
                FROM room_invitations i
                INNER JOIN chat_rooms r ON i.room_id = r.id
                INNER JOIN users u ON i.invited_by = u.id
                WHERE i.user_id = ? AND i.status = ?
                ORDER BY i.created_at DESC
                "#
            )
            .bind(user_id)
            .bind(status)
        } else {
            sqlx::query(
                r#"
                SELECT 
                    i.id, i.room_id, i.invited_by, i.status, i.created_at,
                    r.name as room_name, r.room_type,
                    u.name as invited_by_name
                FROM room_invitations i
                INNER JOIN chat_rooms r ON i.room_id = r.id
                INNER JOIN users u ON i.invited_by = u.id
                WHERE i.user_id = ?
                ORDER BY i.created_at DESC
                "#
            )
            .bind(user_id)
        };

        let rows = query.fetch_all(&self.pool).await?;

        let invitations: Vec<InvitationResponse> = rows
            .iter()
            .map(|row| InvitationResponse {
                id: row.get("id"),
                room_id: row.get("room_id"),
                room_name: row.get("room_name"),
                room_type: row.get("room_type"),
                invited_by: row.get("invited_by"),
                invited_by_name: row.get("invited_by_name"),
                status: row.get("status"),
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(invitations)
    }

    pub async fn update_invitation_status(
        &self,
        invitation_id: i64,
        status: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE room_invitations
            SET status = ?, updated_at = NOW()
            WHERE id = ?
            "#
        )
        .bind(status)
        .bind(invitation_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn has_pending_invitation(
        &self,
        room_id: &str,
        user_id: i64,
    ) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM room_invitations
            WHERE room_id = ? AND user_id = ? AND status = 'pending'
            "#
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    pub async fn is_member_or_invited(
        &self,
        room_id: &str,
        user_id: i64,
    ) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM chat_room_members WHERE room_id = ? AND user_id = ?) +
                (SELECT COUNT(*) FROM room_invitations WHERE room_id = ? AND user_id = ? AND status = 'pending')
                as count
            "#
        )
        .bind(room_id)
        .bind(user_id)
        .bind(room_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }
}
