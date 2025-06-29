// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalProduct {
    pub id: Option<i32>,
    pub original_product_id: i32,
    pub temporal_ticket_id: i32,
    pub name: String,
    pub quantity: i32,
    pub price: Option<f32>,

    // Not in the db
    pub price_input: String, // Helps us input prices on TextInputs
}

impl TemporalProduct {
    pub async fn edit(
        pool: Arc<PgPool>,
        temporal_product: TemporalProduct,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE temporal_products SET quantity = $1, price = $2 WHERE id = $3")
            .bind(temporal_product.quantity)
            .bind(temporal_product.price)
            .bind(temporal_product.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: Arc<PgPool>, temporal_product_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM temporal_products WHERE id = $1")
            .bind(temporal_product_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}
