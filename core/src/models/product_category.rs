// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::{fmt, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProductCategory {
    pub id: Option<i32>,
    pub name: String,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[allow(clippy::derivable_impls)]
impl Default for ProductCategory {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),

            is_deleted: false,
            created_at: Default::default(),
            updated_at: Default::default(),
        }
    }
}

impl fmt::Display for ProductCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl ProductCategory {
    /// Returns true if the entity is valid (ready for submission to the db)
    pub fn is_valid(&self) -> bool {
        if self.name.is_empty() {
            return false;
        }

        true
    }

    pub async fn get_all(pool: Arc<PgPool>) -> Result<Vec<ProductCategory>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, is_deleted, created_at, updated_at FROM product_categories WHERE is_deleted = $1 ORDER BY id ASC",
        )
        .bind(false)
        .fetch_all(pool.as_ref()).await?;

        let mut result = Vec::<ProductCategory>::new();

        for row in rows {
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

    pub async fn get_single(
        pool: Arc<PgPool>,
        product_category_id: i32,
    ) -> Result<ProductCategory, sqlx::Error> {
        let row = sqlx::query(
            "SELECT 
                product_categories.id, 
                product_categories.name, 
                product_categories.is_deleted, 
                product_categories.created_at, 
                product_categories.updated_at
            FROM product_categories 
            WHERE product_categories.id = $1",
        )
        .bind(product_category_id)
        .fetch_one(pool.as_ref())
        .await?;

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

        Ok(product_category)
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
