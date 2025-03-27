// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;

use super::{sold_product::SoldProduct, temporal_ticket::TemporalTicket};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleInvoice {
    pub id: Option<i32>,
    pub payment_method_id: i32,
    pub products: Vec<SoldProduct>,
    pub paid: bool,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl SimpleInvoice {
    /// Creates a simple invoice given a temporal ticket, returns the newly created invoice
    pub async fn create_from_temporal_ticket(
        pool: Arc<PgPool>,
        temporal_ticket: TemporalTicket,
    ) -> Result<SimpleInvoice, sqlx::Error> {
        let mut transaction: Transaction<Postgres> = pool.begin().await?;

        // Insert a new simple_invoice
        let invoice = sqlx::query!(
            r#"
            INSERT INTO simple_invoices (payment_method_id, paid, is_deleted)
            VALUES ($1, FALSE, FALSE)
            RETURNING id, payment_method_id, paid, is_deleted, created_at, updated_at
            "#,
            1 // assume payment method is 1
        )
        .fetch_one(&mut *transaction)
        .await?;

        // insert the products
        for product in &temporal_ticket.products {
            sqlx::query!(
                r#"
                INSERT INTO sold_products (simple_invoice_id, original_product_id, price)
                VALUES ($1, $2, $3)
                "#,
                invoice.id,
                product.original_product_id,
                product.price
            )
            .execute(&mut *transaction)
            .await?;
        }

        // add invoice id to temporal ticket
        sqlx::query!(
            r#"
            UPDATE temporal_tickets
            SET simple_invoice_id = $1
            WHERE id = $2
            "#,
            invoice.id,
            temporal_ticket.id
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(SimpleInvoice {
            id: Some(invoice.id),
            payment_method_id: invoice.payment_method_id,
            products: temporal_ticket
                .products
                .into_iter()
                .map(|tp| SoldProduct {
                    id: None,
                    simple_invoice_id: invoice.id,
                    original_product_id: tp.original_product_id,
                    price: tp.price,
                })
                .collect(),
            paid: invoice.paid,
            is_deleted: invoice.is_deleted,
            created_at: invoice.created_at,
            updated_at: invoice.updated_at,
        })
    }

    /// Gets a simple invoice and its products given the id
    pub async fn get_single(
        pool: Arc<PgPool>,
        simple_invoice_id: i32,
    ) -> Result<SimpleInvoice, sqlx::Error> {
        let invoice = sqlx::query!(
            r#"
            SELECT id, payment_method_id, paid, is_deleted, created_at, updated_at
            FROM simple_invoices
            WHERE id = $1
            "#,
            simple_invoice_id
        )
        .fetch_one(&*pool)
        .await?;

        let sold_products = sqlx::query!(
            r#"
            SELECT id, simple_invoice_id, original_product_id, price
            FROM sold_products
            WHERE simple_invoice_id = $1
            "#,
            simple_invoice_id
        )
        .fetch_all(&*pool)
        .await?
        .into_iter()
        .map(|row| SoldProduct {
            id: Some(row.id),
            simple_invoice_id: row.simple_invoice_id,
            original_product_id: row.original_product_id,
            price: row.price,
        })
        .collect();

        Ok(SimpleInvoice {
            id: Some(invoice.id),
            payment_method_id: invoice.payment_method_id,
            products: sold_products,
            paid: invoice.paid,
            is_deleted: invoice.is_deleted,
            created_at: invoice.created_at,
            updated_at: invoice.updated_at,
        })
    }
}
