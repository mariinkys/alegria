// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

use super::{client::Client, simple_invoice::SimpleInvoice};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoldRoom {
    pub id: Option<i32>,
    pub room_id: Option<i32>,
    pub guests: Vec<Client>,
    pub price: Option<f32>,
    pub invoices: Vec<SimpleInvoice>,
}
