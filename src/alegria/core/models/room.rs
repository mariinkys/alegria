// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::{fmt, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Room {
    pub id: Option<i32>,
    pub room_type_id: Option<i32>,
    pub name: String,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,

    // Not in the db
    pub room_type_name: String, // Helps us JOIN adn return the room type name of the selected room_type_id
    pub default_room_price: Option<f32>, // Helps us JOIN the room_type_id and return the default price for this room
}

#[allow(clippy::derivable_impls)]
impl Default for Room {
    fn default() -> Self {
        Self {
            id: None,
            room_type_id: None,
            name: String::new(),
            is_deleted: false,
            created_at: Default::default(),
            updated_at: Default::default(),
            room_type_name: String::new(),
            default_room_price: None,
        }
    }
}

impl fmt::Display for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Room {
    pub async fn get_all(pool: Arc<PgPool>) -> Result<Vec<Room>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT 
                rooms.id, 
                rooms.room_type_id, 
                rooms.name, 
                rooms.is_deleted, 
                rooms.created_at, 
                rooms.updated_at,
                room_types.name as room_type_name,
                room_types.price as default_room_price 
            FROM rooms 
            LEFT JOIN room_types ON rooms.room_type_id = room_types.id 
            WHERE rooms.is_deleted = $1 
            ORDER BY rooms.id ASC",
        )
        .bind(false)
        .fetch_all(pool.as_ref())
        .await?;

        let mut result = Vec::<Room>::new();

        for row in rows {
            let id: Option<i32> = row.try_get("id")?;
            let room_type_id: Option<i32> = row.try_get("room_type_id")?;
            let name: String = row.try_get("name")?;
            let is_deleted: bool = row.try_get("is_deleted")?;
            let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
            let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;
            let room_type_name: String = row.try_get("room_type_name").unwrap_or_default();
            let default_room_price: Option<f32> = row.try_get("default_room_price").unwrap_or(None);

            let room = Room {
                id,
                room_type_id,
                name,
                is_deleted,
                created_at,
                updated_at,
                room_type_name,
                default_room_price,
            };
            result.push(room);
        }
        Ok(result)
    }

    pub async fn add(pool: Arc<PgPool>, room: Room) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO rooms (name, room_type_id) VALUES ($1, $2)")
            .bind(room.name)
            .bind(room.room_type_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn edit(pool: Arc<PgPool>, room: Room) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE rooms SET name = $1, room_type_id = $2 WHERE id = $3")
            .bind(room.name)
            .bind(room.room_type_id)
            .bind(room.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: Arc<PgPool>, room_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE rooms SET is_deleted = $1 WHERE id = $2")
            .bind(true)
            .bind(room_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}
