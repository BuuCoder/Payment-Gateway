use crate::domain::{Room, RoomMember};
use anyhow::Result;
use sqlx::{MySqlPool, Row};

#[derive(Clone)]
pub struct RoomRepository {
    pub pool: MySqlPool,
}

impl RoomRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_room(&self, room: &Room) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO chat_rooms (id, name, room_type, created_by, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&room.id)
        .bind(&room.name)
        .bind(&room.room_type)
        .bind(room.created_by)
        .bind(room.created_at)
        .bind(room.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_member(&self, room_id: &str, user_id: i64, role: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO chat_room_members (room_id, user_id, role, joined_at)
            VALUES (?, ?, ?, NOW())
            "#
        )
        .bind(room_id)
        .bind(user_id)
        .bind(role)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_room(&self, room_id: &str) -> Result<Option<Room>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, room_type, created_by, created_at, updated_at
            FROM chat_rooms
            WHERE id = ?
            "#
        )
        .bind(room_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Room {
            id: row.get("id"),
            name: row.get("name"),
            room_type: row.get("room_type"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    pub async fn get_room_members(&self, room_id: &str) -> Result<Vec<RoomMember>> {
        let rows = sqlx::query(
            r#"
            SELECT m.id, m.room_id, m.user_id, m.role, m.joined_at
            FROM chat_room_members m
            WHERE m.room_id = ?
            "#
        )
        .bind(room_id)
        .fetch_all(&self.pool)
        .await?;

        let members: Vec<RoomMember> = rows
            .iter()
            .map(|row| RoomMember {
                id: row.get("id"),
                room_id: row.get("room_id"),
                user_id: row.get("user_id"),
                role: row.get("role"),
                joined_at: row.get("joined_at"),
            })
            .collect();

        Ok(members)
    }

    pub async fn get_room_members_with_users(&self, room_id: &str) -> Result<Vec<(RoomMember, Option<String>, Option<String>)>> {
        let rows = sqlx::query(
            r#"
            SELECT m.id, m.room_id, m.user_id, m.role, m.joined_at,
                   u.name as user_name, u.email as user_email
            FROM chat_room_members m
            LEFT JOIN users u ON m.user_id = u.id
            WHERE m.room_id = ?
            "#
        )
        .bind(room_id)
        .fetch_all(&self.pool)
        .await?;

        let members: Vec<(RoomMember, Option<String>, Option<String>)> = rows
            .iter()
            .map(|row| {
                let member = RoomMember {
                    id: row.get("id"),
                    room_id: row.get("room_id"),
                    user_id: row.get("user_id"),
                    role: row.get("role"),
                    joined_at: row.get("joined_at"),
                };
                let user_name: Option<String> = row.try_get("user_name").ok();
                let user_email: Option<String> = row.try_get("user_email").ok();
                (member, user_name, user_email)
            })
            .collect();

        Ok(members)
    }

    pub async fn get_user_rooms(&self, user_id: i64) -> Result<Vec<Room>> {
        let rows = sqlx::query(
            r#"
            SELECT r.id, r.name, r.room_type, r.created_by, r.created_at, r.updated_at
            FROM chat_rooms r
            INNER JOIN chat_room_members m ON r.id = m.room_id
            WHERE m.user_id = ?
            ORDER BY r.updated_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let rooms: Vec<Room> = rows
            .iter()
            .map(|row| Room {
                id: row.get("id"),
                name: row.get("name"),
                room_type: row.get("room_type"),
                created_by: row.get("created_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(rooms)
    }

    pub async fn get_user_rooms_with_members(&self, user_id: i64) -> Result<Vec<(Room, Vec<(RoomMember, Option<String>, Option<String>)>)>> {
        // Get all rooms for user
        let rooms = self.get_user_rooms(user_id).await?;
        
        let mut result = Vec::new();
        for room in rooms {
            let members = self.get_room_members_with_users(&room.id).await?;
            result.push((room, members));
        }
        
        Ok(result)
    }

    pub async fn is_member(&self, room_id: &str, user_id: i64) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM chat_room_members
            WHERE room_id = ? AND user_id = ?
            "#
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    pub async fn find_direct_room(&self, user1_id: i64, user2_id: i64) -> Result<Option<Room>> {
        let row = sqlx::query(
            r#"
            SELECT DISTINCT r.id, r.name, r.room_type, r.created_by, r.created_at, r.updated_at
            FROM chat_rooms r
            INNER JOIN chat_room_members m1 ON r.id = m1.room_id
            INNER JOIN chat_room_members m2 ON r.id = m2.room_id
            WHERE r.room_type = 'direct'
            AND m1.user_id = ?
            AND m2.user_id = ?
            "#
        )
        .bind(user1_id)
        .bind(user2_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Room {
            id: row.get("id"),
            name: row.get("name"),
            room_type: row.get("room_type"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }
}
