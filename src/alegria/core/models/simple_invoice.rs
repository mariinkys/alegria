// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;

use super::sold_product::SoldProduct;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleInvoice {
    pub id: Option<i32>,
    pub products: Vec<SoldProduct>,
    pub paid: bool,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl SimpleInvoice {}
