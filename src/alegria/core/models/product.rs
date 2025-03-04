// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Option<i32>,
    pub category_id: Option<i32>,
    pub name: String,
    pub inside_price: Option<f32>,
    pub outside_price: Option<f32>,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Product {
    pub async fn get_all_by_category(
        pool: Arc<Pool<Sqlite>>,
        category_id: i32,
    ) -> Result<Vec<Product>, sqlx::Error> {
        let mut rows = sqlx::query(
            "SELECT id, category_id, name, inside_price, outside_price, is_deleted, created_at, updated_at FROM products WHERE category_id = $1 ORDER BY id ASC",
        )
        .bind(category_id)
        .fetch(pool.as_ref());

        let mut result = Vec::<Product>::new();

        while let Some(row) = rows.try_next().await? {
            let id: Option<i32> = row.try_get("id")?;
            let category_id: Option<i32> = row.try_get("category_id")?;
            let name: String = row.try_get("name")?;
            let inside_price: Option<f32> = row.try_get("inside_price")?;
            let outside_price: Option<f32> = row.try_get("outside_price")?;
            let is_deleted: bool = row.try_get("is_deleted")?;
            let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
            let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;

            let product = Product {
                id,
                category_id,
                name,
                inside_price,
                outside_price,
                is_deleted,
                created_at,
                updated_at,
            };

            result.push(product);
        }

        Ok(result)
    }

    pub async fn add(pool: Arc<Pool<Sqlite>>, product: Product) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO products (category_id, name, inside_price, outside_price, is_deleted) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(product.category_id)
        .bind(product.name)
        .bind(product.inside_price)
        .bind(product.outside_price)
        .bind(false)
        .execute(pool.as_ref())
        .await?;

        Ok(())
    }

    pub async fn edit(pool: Arc<Pool<Sqlite>>, product: Product) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE products SET category_id = $1, name = $2, inside_price = $3, outside_price = $4 WHERE id = $5")
            .bind(product.category_id)
            .bind(product.name)
            .bind(product.inside_price)
            .bind(product.outside_price)
            .bind(product.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: Arc<Pool<Sqlite>>, product_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE products SET is_deleted = $1 WHERE id = $2")
            .bind(true)
            .bind(product_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}
