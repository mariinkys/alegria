// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::{fmt, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: Option<i32>,
    pub category_id: Option<i32>,
    pub name: String,
    pub inside_price: Option<f32>,
    pub outside_price: Option<f32>,
    pub tax_percentage: Option<f32>,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,

    // Not in the db
    #[sqlx(default)]
    pub product_category_name: Box<str>, // Helps us JOIN adn return the product category name of the selected product_category_id
    #[sqlx(default)]
    pub inside_price_input: String, // Helps us input on TextInputs
    #[sqlx(default)]
    pub outside_price_input: String, // Helps us input on TextInputs
    #[sqlx(default)]
    pub tax_percentage_input: String, // Helps us input on TextInputs
}

#[allow(clippy::derivable_impls)]
impl Default for Product {
    fn default() -> Self {
        Self {
            id: None,
            category_id: None,
            name: String::new(),
            inside_price: None,
            outside_price: None,
            tax_percentage: None,
            is_deleted: false,
            created_at: Default::default(),
            updated_at: Default::default(),
            product_category_name: String::new().into_boxed_str(),
            inside_price_input: String::new(),
            outside_price_input: String::new(),
            tax_percentage_input: String::new(),
        }
    }
}

impl fmt::Display for Product {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Product {
    /// Returns true if the entity is valid (ready for submission to the db)
    pub fn is_valid(&self) -> bool {
        if self.name.is_empty()
            || self.inside_price.is_none()
            || self.outside_price.is_none()
            || self.tax_percentage.is_none()
            || self.category_id.is_none()
        {
            return false;
        }

        true
    }

    pub async fn get_all(pool: Arc<PgPool>) -> Result<Vec<Product>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT 
                products.id, 
                products.category_id, 
                products.name, 
                products.inside_price, 
                products.outside_price, 
                products.tax_percentage, 
                products.is_deleted, 
                products.created_at, 
                products.updated_at,
                product_categories.name as product_category_name
            FROM products
            LEFT JOIN product_categories ON products.category_id = product_categories.id
            WHERE products.is_deleted = $1 ORDER BY id ASC",
        )
        .bind(false)
        .fetch_all(pool.as_ref())
        .await?;

        let mut result = Vec::<Product>::new();

        for row in rows {
            let id: Option<i32> = row.try_get("id")?;
            let category_id: Option<i32> = row.try_get("category_id")?;
            let name: String = row.try_get("name")?;
            let inside_price: Option<f32> = row.try_get("inside_price")?;
            let outside_price: Option<f32> = row.try_get("outside_price")?;
            let tax_percentage: Option<f32> = row.try_get("tax_percentage")?;
            let is_deleted: bool = row.try_get("is_deleted")?;
            let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
            let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;
            let product_category_name: String =
                row.try_get("product_category_name").unwrap_or_default();

            let product = Product {
                id,
                category_id,
                name,
                inside_price,
                outside_price,
                tax_percentage,
                is_deleted,
                created_at,
                updated_at,
                product_category_name: product_category_name.into_boxed_str(),
                ..Default::default()
            };

            result.push(product);
        }

        Ok(result)
    }

    pub async fn get_all_by_category(
        pool: Arc<PgPool>,
        category_id: i32,
    ) -> Result<Vec<Product>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, category_id, name, inside_price, outside_price, tax_percentage, is_deleted, created_at, updated_at FROM products WHERE category_id = $1 AND is_deleted = $2 ORDER BY id ASC",
        )
        .bind(category_id)
        .bind(false)
        .fetch_all(pool.as_ref()).await?;

        let mut result = Vec::<Product>::new();

        for row in rows {
            let id: Option<i32> = row.try_get("id")?;
            let category_id: Option<i32> = row.try_get("category_id")?;
            let name: String = row.try_get("name")?;
            let inside_price: Option<f32> = row.try_get("inside_price")?;
            let outside_price: Option<f32> = row.try_get("outside_price")?;
            let tax_percentage: Option<f32> = row.try_get("tax_percentage")?;
            let is_deleted: bool = row.try_get("is_deleted")?;
            let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
            let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;

            let product = Product {
                id,
                category_id,
                name,
                inside_price,
                outside_price,
                tax_percentage,
                is_deleted,
                created_at,
                updated_at,
                product_category_name: String::new().into_boxed_str(),
                ..Default::default()
            };

            result.push(product);
        }

        Ok(result)
    }

    pub async fn get_single(pool: Arc<PgPool>, product_id: i32) -> Result<Product, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, 
                category_id, 
                name, 
                inside_price, 
                outside_price, 
                tax_percentage, 
                is_deleted, 
                created_at, 
                updated_at
            FROM products 
            WHERE products.id = $1",
        )
        .bind(product_id)
        .fetch_one(pool.as_ref())
        .await?;

        let id: Option<i32> = row.try_get("id")?;
        let category_id: Option<i32> = row.try_get("category_id")?;
        let name: String = row.try_get("name")?;
        let inside_price: Option<f32> = row.try_get("inside_price")?;
        let outside_price: Option<f32> = row.try_get("outside_price")?;
        let tax_percentage: Option<f32> = row.try_get("tax_percentage")?;
        let is_deleted: bool = row.try_get("is_deleted")?;
        let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
        let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;

        let product = Product {
            id,
            category_id,
            name,
            inside_price,
            outside_price,
            tax_percentage,
            is_deleted,
            created_at,
            updated_at,
            product_category_name: String::new().into_boxed_str(),
            inside_price_input: inside_price.map_or(String::new(), |p| format!("{p:.2}")),
            outside_price_input: outside_price.map_or(String::new(), |p| format!("{p:.2}")),
            tax_percentage_input: tax_percentage.map_or(String::new(), |p| format!("{p:.2}")),
        };

        Ok(product)
    }

    pub async fn add(pool: Arc<PgPool>, product: Product) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO products (category_id, name, inside_price, outside_price, tax_percentage, is_deleted) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(product.category_id)
        .bind(product.name)
        .bind(product.inside_price)
        .bind(product.outside_price)
        .bind(product.tax_percentage)
        .bind(false)
        .execute(pool.as_ref())
        .await?;

        Ok(())
    }

    pub async fn edit(pool: Arc<PgPool>, product: Product) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE products SET category_id = $1, name = $2, inside_price = $3, outside_price = $4, tax_percentage = $5 WHERE id = $6")
            .bind(product.category_id)
            .bind(product.name)
            .bind(product.inside_price)
            .bind(product.outside_price)
            .bind(product.tax_percentage)
            .bind(product.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: Arc<PgPool>, product_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE products SET is_deleted = $1 WHERE id = $2")
            .bind(true)
            .bind(product_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}
