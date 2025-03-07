// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::{collections::HashMap, sync::Arc};

use super::{product::Product, temporal_product::TemporalProduct};

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

        // Sort the products for each ticket by product id
        for ticket in tickets_map.values_mut() {
            ticket.products.sort_by_key(|p| p.id);
        }

        // Collect the tickets and sort them by id
        let mut tickets: Vec<TemporalTicket> = tickets_map.into_values().collect();
        tickets.sort_by_key(|t| t.id);
        Ok(tickets)
    }

    pub async fn upsert_ticket_by_id_and_tableloc(
        pool: Arc<Pool<Sqlite>>,
        temporal_ticket: TemporalTicket,
        new_product_id: i32,
    ) -> Result<(), sqlx::Error> {
        let product_row =
            sqlx::query("SELECT id, category_id, name, inside_price, outside_price, is_deleted, created_at, updated_at FROM products WHERE id = ?")
                .bind(new_product_id)
                .fetch_optional(pool.as_ref())
                .await?;

        let product: Product = match product_row {
            Some(row) => {
                let id: Option<i32> = row.try_get("id")?;
                let category_id: Option<i32> = row.try_get("category_id")?;
                let name: String = row.try_get("name")?;
                let inside_price: Option<f32> = row.try_get("inside_price")?;
                let outside_price: Option<f32> = row.try_get("outside_price")?;
                let is_deleted: bool = row.try_get("is_deleted")?;
                let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
                let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;

                Product {
                    id,
                    category_id,
                    name,
                    inside_price,
                    outside_price,
                    is_deleted,
                    created_at,
                    updated_at,
                }
            }
            None => return Err(sqlx::Error::RowNotFound),
        };

        // Check if a ticket already exists with the same table_id and ticket_location.
        let existing_ticket = sqlx::query(
            "SELECT id FROM temporal_tickets WHERE table_id = ? AND ticket_location = ?",
        )
        .bind(temporal_ticket.table_id)
        .bind(temporal_ticket.ticket_location)
        .fetch_optional(pool.as_ref())
        .await?;

        // TODO: Should use a transaction

        // Check if the ticket already exists; if not, insert a new temporal_ticket.
        let ticket_id = if let Some(row) = existing_ticket {
            row.try_get("id")?
        } else {
            let result = sqlx::query(
                "INSERT INTO temporal_tickets (table_id, ticket_location, ticket_status) VALUES (?, ?, ?)"
            )
            .bind(temporal_ticket.table_id)
            .bind(temporal_ticket.ticket_location)
            .bind(temporal_ticket.ticket_status)
            .execute(pool.as_ref())
            .await?;

            // retrieve the last inserted row ID.
            result.last_insert_rowid() as i32
        };

        sqlx::query(
            "INSERT INTO temporal_products (original_product_id, temporal_ticket_id, quantity, name, price) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(product.id)
        .bind(ticket_id)
        .bind(1) // quantity is hard-coded as 1
        .bind(product.name)
        .bind(
            if temporal_ticket.ticket_location == 1 {
                product.inside_price
            } else {
                product.outside_price
            }
        )
        .execute(pool.as_ref())
        .await?;

        Ok(())
    }

    pub async fn delete(
        pool: Arc<Pool<Sqlite>>,
        temporal_ticket_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM temporal_tickets WHERE id = ?")
            .bind(temporal_ticket_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}
