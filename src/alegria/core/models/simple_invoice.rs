// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;

use crate::alegria::core::models::product::Product;

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

        let mut sold_products = Vec::new();

        // insert the products and fetch og_product details
        for product in &temporal_ticket.products {
            let sold_product = sqlx::query!(
                r#"
                INSERT INTO sold_products (simple_invoice_id, original_product_id, price)
                VALUES ($1, $2, $3)
                RETURNING id, simple_invoice_id, original_product_id, price
                "#,
                invoice.id,
                product.original_product_id,
                product.price
            )
            .fetch_one(&mut *transaction)
            .await?;

            let original_product = sqlx::query_as!(
                Product,
                r#"
                SELECT id, category_id, name, inside_price, outside_price, tax_percentage, is_deleted, created_at, updated_at
                FROM products
                WHERE id = $1
                "#,
                sold_product.original_product_id
            )
            .fetch_one(&mut *transaction)
            .await?;

            sold_products.push(SoldProduct {
                id: Some(sold_product.id),
                simple_invoice_id: sold_product.simple_invoice_id,
                original_product_id: sold_product.original_product_id,
                price: sold_product.price,
                original_product,
            });
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
            products: sold_products,
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
            SELECT sp.id, sp.simple_invoice_id, sp.original_product_id, sp.price,
                   p.id as "p_id", p.category_id as "p_category_id", p.name as "p_name",
                   p.inside_price as "p_inside_price", p.outside_price as "p_outside_price",
                   p.tax_percentage as "p_tax_percentage", p.is_deleted as "p_is_deleted",
                   p.created_at as "p_created_at", p.updated_at as "p_updated_at"
            FROM sold_products sp
            JOIN products p ON sp.original_product_id = p.id
            WHERE sp.simple_invoice_id = $1
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
            original_product: Product {
                id: Some(row.p_id),
                category_id: row.p_category_id,
                name: row.p_name,
                inside_price: row.p_inside_price,
                outside_price: row.p_outside_price,
                tax_percentage: row.p_tax_percentage,
                is_deleted: row.p_is_deleted,
                created_at: row.p_created_at,
                updated_at: row.p_updated_at,
            },
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

    /// Deletes a simple invoice given a TemporalTicket
    pub async fn unlock_temporal_ticket(
        pool: Arc<PgPool>,
        temporal_ticket: TemporalTicket,
    ) -> Result<(), sqlx::Error> {
        let mut transaction: Transaction<Postgres> = pool.begin().await?;

        // Retrieve the simple_invoice_id associated with the temporal_ticket
        let invoice = sqlx::query!(
            r#"
            SELECT simple_invoice_id FROM temporal_tickets WHERE id = $1
            "#,
            temporal_ticket.id
        )
        .fetch_one(&mut *transaction)
        .await?;

        if let Some(invoice_id) = invoice.simple_invoice_id {
            // delete the simple_invoice (the db cascade will handle sold_products deletion)
            sqlx::query!(
                r#"
                DELETE FROM simple_invoices WHERE id = $1
                "#,
                invoice_id
            )
            .execute(&mut *transaction)
            .await?;
        }

        // update the temporal_ticket to remove the invoice reference
        sqlx::query!(
            r#"
            UPDATE temporal_tickets SET simple_invoice_id = NULL WHERE id = $1
            "#,
            temporal_ticket.id
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }
}
