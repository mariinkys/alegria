// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: Option<i32>,
    pub room_type_id: i32,
    pub name: String,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
