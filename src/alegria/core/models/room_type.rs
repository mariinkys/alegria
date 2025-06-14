// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::{fmt, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoomType {
    pub id: Option<i32>,
    pub name: String,
    pub price: Option<f32>,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,

    // Not in the db
    pub price_input: String, // Helps us input prices on TextInputs
}

#[allow(clippy::derivable_impls)]
impl Default for RoomType {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            price: None,
            is_deleted: false,
            created_at: Default::default(),
            updated_at: Default::default(),
            price_input: String::new(),
        }
    }
}

impl fmt::Display for RoomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl RoomType {
    /// Returns true if the client is valid (ready for submission to the db)
    pub fn is_valid(&self) -> bool {
        if self.name.is_empty() || self.price_input.is_empty() || self.price.is_none() {
            return false;
        }

        true
    }

    pub async fn get_all(pool: Arc<PgPool>) -> Result<Vec<RoomType>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, price, is_deleted, created_at, updated_at FROM room_types WHERE is_deleted = $1 ORDER BY id ASC",
        )
        .bind(false)
        .fetch_all(pool.as_ref()).await?;

        let mut result = Vec::<RoomType>::new();

        for row in rows {
            let id: Option<i32> = row.try_get("id")?;
            let name: String = row.try_get("name")?;
            let price: Option<f32> = row.try_get("price")?;
            let is_deleted: bool = row.try_get("is_deleted")?;
            let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
            let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;

            let room_type = RoomType {
                id,
                name,
                price,
                is_deleted,
                created_at,
                updated_at,
                price_input: price.unwrap_or(0.0).to_string(),
            };

            result.push(room_type);
        }

        Ok(result)
    }

    pub async fn get_single(pool: Arc<PgPool>, room_type_id: i32) -> Result<RoomType, sqlx::Error> {
        let row = sqlx::query(
            "SELECT 
                room_types.id, 
                room_types.name, 
                room_types.price, 
                room_types.is_deleted, 
                room_types.created_at, 
                room_types.updated_at
            FROM room_types 
            WHERE room_types.id = $1",
        )
        .bind(room_type_id)
        .fetch_one(pool.as_ref())
        .await?;

        let id: Option<i32> = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let price: Option<f32> = row.try_get("price")?;
        let is_deleted: bool = row.try_get("is_deleted")?;
        let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
        let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;

        let room_type = RoomType {
            id,
            name,
            price,
            is_deleted,
            created_at,
            updated_at,
            price_input: price.map_or(String::new(), |p| format!("{p:.2}")),
        };

        Ok(room_type)
    }

    pub async fn add(pool: Arc<PgPool>, room_type: RoomType) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO room_types (name, price) VALUES ($1, $2)")
            .bind(room_type.name)
            .bind(room_type.price)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn edit(pool: Arc<PgPool>, room_type: RoomType) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE room_types SET name = $1, price = $2 WHERE id = $3")
            .bind(room_type.name)
            .bind(room_type.price)
            .bind(room_type.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: Arc<PgPool>, room_type_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE room_types SET is_deleted = $1 WHERE id = $2")
            .bind(true)
            .bind(room_type_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}
