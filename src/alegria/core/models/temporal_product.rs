// SPDX-License-Identifier: GPL-3.0-only

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalProduct {
    pub id: Option<i32>,
    pub original_product_id: i32,
    pub temporal_ticket_id: i32,
    pub name: String,
    pub quantity: i32,
    pub price: Option<f32>,
}

impl TemporalProduct {}
