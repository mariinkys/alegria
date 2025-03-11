// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomType {
    pub id: Option<i32>,
    pub name: String,
    pub price: Option<f32>,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl RoomType {
    pub async fn get_all(pool: Arc<Pool<Sqlite>>) -> Result<Vec<RoomType>, sqlx::Error> {
        let mut rows = sqlx::query(
            "SELECT id, name, price, is_deleted, created_at, updated_at FROM room_types ORDER BY id ASC",
        )
        .fetch(pool.as_ref());

        let mut result = Vec::<RoomType>::new();

        while let Some(row) = rows.try_next().await? {
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
            };

            result.push(room_type);
        }

        Ok(result)
    }
}
