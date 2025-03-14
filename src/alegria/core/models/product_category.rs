// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductCategory {
    pub id: Option<i32>,
    pub name: String,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl ProductCategory {
    pub async fn get_all(pool: Arc<PgPool>) -> Result<Vec<ProductCategory>, sqlx::Error> {
        let mut rows = sqlx::query(
            "SELECT id, name, is_deleted, created_at, updated_at FROM product_categories ORDER BY id ASC",
        )
        .fetch(pool.as_ref());

        let mut result = Vec::<ProductCategory>::new();

        while let Some(row) = rows.try_next().await? {
            let id: Option<i32> = row.try_get("id")?;
            let name: String = row.try_get("name")?;
            let is_deleted: bool = row.try_get("is_deleted")?;
            let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
            let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;

            let product_category = ProductCategory {
                id,
                name,
                is_deleted,
                created_at,
                updated_at,
            };

            result.push(product_category);
        }

        Ok(result)
    }

    pub async fn add(
        pool: Arc<PgPool>,
        product_category: ProductCategory,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO product_categories (name, is_deleted) VALUES ($1, $2)")
            .bind(product_category.name)
            .bind(false)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn edit(
        pool: Arc<PgPool>,
        product_category: ProductCategory,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE product_categories SET name = $1 WHERE id = $2")
            .bind(product_category.name)
            .bind(product_category.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: Arc<PgPool>, product_category_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE product_categories SET is_deleted = $1 WHERE id = $2")
            .bind(true)
            .bind(product_category_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}
