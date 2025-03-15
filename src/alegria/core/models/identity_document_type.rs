// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdentityDocumentType {
    pub id: Option<i32>,
    pub name: String,
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for IdentityDocumentType {
    fn to_string(&self) -> String {
        self.name.to_string()
    }
}

impl IdentityDocumentType {
    pub async fn get_all(pool: Arc<PgPool>) -> Result<Vec<IdentityDocumentType>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, name FROM identity_document_types ORDER BY id ASC")
            .fetch_all(pool.as_ref())
            .await?;

        let mut result = Vec::<IdentityDocumentType>::new();

        for row in rows {
            let id: Option<i32> = row.try_get("id")?;
            let name: String = row.try_get("name")?;

            let idt = IdentityDocumentType { id, name };

            result.push(idt);
        }

        Ok(result)
    }
}
