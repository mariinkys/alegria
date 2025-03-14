// SPDX-License-Identifier: GPL-3.0-only

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: Option<i32>,
    // TODO: Should IdentityDocument be it's own table?
    pub identity_document_type_id: i32,
    pub identity_document: String,
    pub identity_document_expedition_date: Option<NaiveDateTime>,
    pub identity_document_expiration_date: Option<NaiveDateTime>,
    pub name: String,
    pub first_surname: String,
    pub second_surname: String,
    pub birthdate: Option<NaiveDateTime>,
    pub address: String,
    pub postal_code: String,
    pub city: String,
    pub province: String,
    pub country: String,
    pub nationality: String,
    pub phone_number: String,
    pub mobile_phone: String,
    pub gender: bool,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
