// SPDX-License-Identifier: GPL-3.0-only

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::{collections::HashMap, sync::Arc};

use super::temporal_product::TemporalProduct;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalTicket {
    pub id: Option<i32>,
    pub table_id: i32,
    pub ticket_location: i32,
    pub ticket_status: i32,
    pub products: Vec<TemporalProduct>,
}

impl TemporalTicket {
    pub async fn get_all(pool: Arc<Pool<Sqlite>>) -> Result<Vec<TemporalTicket>, sqlx::Error> {
        let mut rows = sqlx::query(
            "SELECT 
                t.id as ticket_id,
                t.table_id,
                t.ticket_location,
                t.ticket_status,
                p.id as product_id,
                p.original_product_id,
                p.temporal_ticket_id,
                p.name as product_name,
                p.quantity as product_quantity,
                p.price as product_price
             FROM temporal_tickets t
             LEFT JOIN temporal_products p ON p.temporal_ticket_id = t.id
             ORDER BY t.id ASC",
        )
        .fetch(pool.as_ref());

        // group rows by ticket id
        let mut tickets_map: HashMap<i32, TemporalTicket> = HashMap::new();

        while let Some(row) = rows.try_next().await? {
            let ticket_id: i32 = row.try_get("ticket_id")?;
            let table_id: i32 = row.try_get("table_id")?;
            let ticket_location: i32 = row.try_get("ticket_location")?;
            let ticket_status: i32 = row.try_get("ticket_status")?;

            // Insert the ticket into the map if it doesn't exist yet
            let ticket = tickets_map.entry(ticket_id).or_insert(TemporalTicket {
                id: Some(ticket_id),
                table_id,
                ticket_location,
                ticket_status,
                products: Vec::new(),
            });

            // Check if a product is attached to this row
            let product_id: Option<i32> = row.try_get("product_id")?;
            if let Some(pid) = product_id {
                let original_product_id: i32 = row.try_get("original_product_id")?;
                let temporal_ticket_id: i32 = row.try_get("temporal_ticket_id")?;
                let product_name: String = row.try_get("product_name")?;
                let product_quantity: i32 = row.try_get("product_quantity")?;
                let product_price: Option<f32> = row.try_get("product_price")?;

                let product = TemporalProduct {
                    id: Some(pid),
                    original_product_id,
                    temporal_ticket_id,
                    name: product_name,
                    quantity: product_quantity,
                    price: product_price,
                };

                ticket.products.push(product);
            }
        }

        // Collect the tickets and sort them by id
        let mut tickets: Vec<TemporalTicket> =
            tickets_map.into_iter().map(|(_, ticket)| ticket).collect();
        tickets.sort_by_key(|t| t.id);
        Ok(tickets)
    }
}
