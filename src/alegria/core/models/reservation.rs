// SPDX-License-Identifier: GPL-3.0-only

use chrono::{Datelike, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;

use super::{simple_invoice::SimpleInvoice, sold_room::SoldRoom};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reservation {
    pub id: Option<i32>,
    pub client_id: Option<i32>,
    pub rooms: Vec<SoldRoom>,
    pub entry_date: Option<NaiveDateTime>,
    pub departure_date: Option<NaiveDateTime>,
    pub room_invoices: Vec<SimpleInvoice>,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,

    // Not in the db
    pub client_name: String, // Helps us JOIN and return the name of the selected client
}

#[allow(clippy::derivable_impls)]
impl Default for Reservation {
    fn default() -> Self {
        Self {
            id: None,
            client_id: None,
            rooms: Vec::new(),
            entry_date: None,
            departure_date: None,
            room_invoices: Vec::new(),
            is_deleted: false,
            created_at: None,
            updated_at: None,
            client_name: String::new(),
        }
    }
}
