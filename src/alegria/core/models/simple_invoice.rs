// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;

use crate::alegria::{
    core::models::product::Product, utils::entities::payment_method::PaymentMethod,
};

use super::{sold_product::SoldProduct, temporal_ticket::TemporalTicket};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleInvoice {
    pub id: Option<i32>,
    pub payment_method: PaymentMethod,
    pub products: Vec<SoldProduct>,
    pub paid: bool,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl SimpleInvoice {
    pub fn total_price(&self) -> f32 {
        let mut price = 0.;
        for product in &self.products {
            price += product.price.unwrap_or(0.);
        }

        price
    }

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
            PaymentMethod::to_id(PaymentMethod::Efectivo) // assume payment method is efectivo
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

            let original_product = sqlx::query_as::<_, Product>(
                r#"
                SELECT id, category_id, name, inside_price, outside_price, tax_percentage, is_deleted, created_at, updated_at
                FROM products
                WHERE id = $1
                "#,
            )
            .bind(sold_product.original_product_id)
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
            payment_method: PaymentMethod::from_id(invoice.payment_method_id).unwrap_or_default(),
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
                product_category_name: String::new().into_boxed_str(),
                ..Default::default()
            },
        })
        .collect();

        Ok(SimpleInvoice {
            id: Some(invoice.id),
            payment_method: PaymentMethod::from_id(invoice.payment_method_id).unwrap_or_default(),
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

    /// Creates a simple invoice given a temporal ticket, returns the newly created invoice
    pub async fn pay_temporal_ticket(
        pool: Arc<PgPool>,
        temporal_ticket_id: i32,
        payment_method: PaymentMethod,
        sold_room_id: Option<i32>, // TODO: If payment method is adeudo assign the ticket to the sold_room
    ) -> Result<(), sqlx::Error> {
        let mut transaction: Transaction<Postgres> = pool.begin().await?;

        // get the temporal ticket with the given id
        let temporal_ticket = sqlx::query!(
            r#"
            SELECT id, simple_invoice_id FROM temporal_tickets WHERE id = $1
            "#,
            temporal_ticket_id
        )
        .fetch_one(&mut *transaction)
        .await?;

        // if the temporal ticket has simple_invoice_id, update it's paid bool and payment_method_id
        if let Some(simple_invoice_id) = temporal_ticket.simple_invoice_id {
            sqlx::query!(
                r#"
                UPDATE simple_invoices SET paid = TRUE, payment_method_id = $1 WHERE id = $2
                "#,
                payment_method.to_id(),
                simple_invoice_id
            )
            .execute(&mut *transaction)
            .await?;

            // if the paymeent method is adeudo add the simple invoice to the sold room
            if payment_method == PaymentMethod::Adeudo {
                if let Some(sold_room_id) = sold_room_id {
                    sqlx::query!(
                        r#"
                            INSERT INTO sold_room_invoices (sold_room_id, simple_invoice_id)
                            VALUES ($1, $2)
                        
                        "#,
                        sold_room_id,
                        simple_invoice_id
                    )
                    .execute(&mut *transaction)
                    .await?;
                } else {
                    eprintln!("Error, adeudo without sold_room_id hit the db");
                    return Err(sqlx::Error::Protocol(
                        "Missing sold_room_id for adeudo hit the db".into(),
                    ));
                }
            }
        } else {
            // if the temporal ticket is not yet a simple_invoice_id create it with the data of the retrieved temporal ticket
            let invoice = sqlx::query!(
                r#"
                INSERT INTO simple_invoices (payment_method_id, paid, is_deleted)
                VALUES ($1, TRUE, FALSE)
                RETURNING id
                "#,
                payment_method.to_id()
            )
            .fetch_one(&mut *transaction)
            .await?;

            // retrieve temporal products associated with the temporal ticket
            let temporal_products = sqlx::query!(
                r#"
                SELECT original_product_id, price FROM temporal_products WHERE temporal_ticket_id = $1
                "#,
                temporal_ticket_id
            )
            .fetch_all(&mut *transaction)
            .await?;

            // insert temporal products into sold_products
            for product in temporal_products {
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

            // if the paymeent method is adeudo add the simple invoice to the sold room
            if payment_method == PaymentMethod::Adeudo {
                if let Some(sold_room_id) = sold_room_id {
                    sqlx::query!(
                        r#"
                            INSERT INTO sold_room_invoices (sold_room_id, simple_invoice_id)
                            VALUES ($1, $2)
                        
                        "#,
                        sold_room_id,
                        invoice.id
                    )
                    .execute(&mut *transaction)
                    .await?;
                } else {
                    eprintln!("Error, adeudo without sold_room_id hit the db");
                    return Err(sqlx::Error::Protocol(
                        "Missing sold_room_id for adeudo hit the db".into(),
                    ));
                }
            }
        }

        // delete the temporal ticket (temporal products will be deleted by on_cascade of the db)
        sqlx::query!(
            r#"
            DELETE FROM temporal_tickets WHERE id = $1
            "#,
            temporal_ticket_id
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    /// Retrieves all simple invoices from the database
    pub async fn get_all(pool: Arc<PgPool>) -> Result<Vec<SimpleInvoice>, sqlx::Error> {
        use sqlx::Row;

        let rows = sqlx::query(
            "SELECT
            si.id,
            si.payment_method_id,
            si.paid,
            si.is_deleted,
            si.created_at,
            si.updated_at,
            sp.id as product_id,
            sp.simple_invoice_id,
            sp.original_product_id,
            sp.price,
            p.id as original_product_id_field,
            p.category_id,
            p.name as product_name,
            p.inside_price,
            p.outside_price,
            p.tax_percentage,
            p.is_deleted as product_is_deleted,
            p.created_at as product_created_at,
            p.updated_at as product_updated_at
            FROM simple_invoices si
            LEFT JOIN sold_products sp ON si.id = sp.simple_invoice_id
            LEFT JOIN products p ON sp.original_product_id = p.id
            WHERE si.is_deleted = $1
            ORDER BY si.id DESC",
        )
        .bind(false)
        .fetch_all(pool.as_ref())
        .await?;

        let mut invoices_map = std::collections::HashMap::<i32, SimpleInvoice>::new();

        for row in rows {
            let id: Option<i32> = row.try_get("id")?;
            let invoice_id = id.unwrap_or(0);

            let invoice = invoices_map.entry(invoice_id).or_insert_with(|| {
                let payment_method_id: Option<i32> =
                    row.try_get("payment_method_id").unwrap_or(None);
                let paid: bool = row.try_get("paid").unwrap_or(false);
                let is_deleted: bool = row.try_get("is_deleted").unwrap_or(false);
                let created_at: Option<NaiveDateTime> = row.try_get("created_at").unwrap_or(None);
                let updated_at: Option<NaiveDateTime> = row.try_get("updated_at").unwrap_or(None);

                SimpleInvoice {
                    id,
                    payment_method: PaymentMethod::from_id(payment_method_id.unwrap_or_default())
                        .unwrap_or_default(),
                    products: Vec::new(),
                    paid,
                    is_deleted,
                    created_at,
                    updated_at,
                }
            });

            let product_id: Option<i32> = row.try_get("product_id")?;
            if let Some(product_id) = product_id {
                let simple_invoice_id: i32 = row.try_get("simple_invoice_id")?;
                let original_product_id: i32 = row.try_get("original_product_id")?;
                let price: Option<f32> = row.try_get("price")?;

                let original_product_id_field: Option<i32> =
                    row.try_get("original_product_id_field")?;
                let category_id: Option<i32> = row.try_get("category_id")?;
                let product_name: String = row.try_get("product_name")?;
                let inside_price: Option<f32> = row.try_get("inside_price")?;
                let outside_price: Option<f32> = row.try_get("outside_price")?;
                let tax_percentage: Option<f32> = row.try_get("tax_percentage")?;
                let product_is_deleted: bool = row.try_get("product_is_deleted")?;
                let product_created_at: Option<NaiveDateTime> =
                    row.try_get("product_created_at")?;
                let product_updated_at: Option<NaiveDateTime> =
                    row.try_get("product_updated_at")?;

                let original_product = Product {
                    id: original_product_id_field,
                    category_id,
                    name: product_name,
                    inside_price,
                    outside_price,
                    tax_percentage,
                    is_deleted: product_is_deleted,
                    created_at: product_created_at,
                    updated_at: product_updated_at,
                    product_category_name: Box::from(""),
                    inside_price_input: String::new(),
                    outside_price_input: String::new(),
                    tax_percentage_input: String::new(),
                };

                let sold_product = SoldProduct {
                    id: Some(product_id),
                    simple_invoice_id,
                    original_product_id,
                    price,
                    original_product,
                };

                invoice.products.push(sold_product);
            }
        }

        let mut result: Vec<SimpleInvoice> = invoices_map.into_values().collect();
        result.sort_by(|a, b| b.id.cmp(&a.id));

        Ok(result)
    }

    /// delete the simple_invoice (the db cascade will handle sold_products deletion, will delete the simple invoice from any adeudo...)
    pub async fn delete(pool: Arc<PgPool>, simple_invoice_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM simple_invoices WHERE id = $1")
            .bind(simple_invoice_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}
