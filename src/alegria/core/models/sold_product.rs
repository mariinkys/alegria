// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

use super::product::Product;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoldProduct {
    pub id: Option<i32>,
    pub simple_invoice_id: i32,
    pub original_product_id: i32,
    pub price: Option<f32>,

    // Not in the db
    pub original_product: Product,
}
