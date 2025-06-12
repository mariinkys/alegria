// SPDX-License-Identifier: GPL-3.0-only

use chrono::{Datelike, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;

use crate::alegria::utils::entities::{
    gender::Gender, identity_document_type::IdentityDocumentType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: Option<i32>,
    pub gender: Option<Gender>,
    pub identity_document_type: Option<IdentityDocumentType>,
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
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,

    // Not in the db
    pub birthdate_string: String, // Helps us input the date as a string
    pub identity_document_expedition_date_string: String, // Helps us input the date as a string
    pub identity_document_expiration_date_string: String, // Helps us input the date as a string
    pub search_field: Box<str>, // Helps us search, this field will have all the data you can search a client for on a string
}

#[allow(clippy::derivable_impls)]
impl Default for Client {
    fn default() -> Self {
        Self {
            id: None,
            gender: Some(Gender::Male), // This makes male the default selected gender
            identity_document_type: Some(IdentityDocumentType::Dni), // This makes DNI the dfault selected type
            identity_document: String::new(),
            identity_document_expedition_date: None,
            identity_document_expiration_date: None,
            name: String::new(),
            first_surname: String::new(),
            second_surname: String::new(),
            birthdate: None,
            address: String::new(),
            postal_code: String::new(),
            city: String::new(),
            province: String::new(),
            country: String::from("España"),
            nationality: String::from("España"),
            phone_number: String::new(),
            mobile_phone: String::new(),
            is_deleted: false,
            created_at: None,
            updated_at: None,

            birthdate_string: String::new(),
            identity_document_expedition_date_string: String::new(),
            identity_document_expiration_date_string: String::new(),
            search_field: String::new().into_boxed_str(),
        }
    }
}

impl Client {
    pub async fn get_all(pool: Arc<PgPool>) -> Result<Vec<Client>, sqlx::Error> {
        // We retrieve only the fields needed for the grid

        let rows = sqlx::query(
            "SELECT 
                clients.id, 
                clients.identity_document_type_id, 
                clients.identity_document, 
                clients.name, 
                clients.first_surname, 
                clients.second_surname, 
                clients.country, 
                clients.is_deleted, 
                clients.created_at, 
                clients.updated_at,
            FROM clients 
            WHERE clients.is_deleted = $1 
            ORDER BY clients.id DESC",
        )
        .bind(false)
        .fetch_all(pool.as_ref())
        .await?;

        let mut result = Vec::<Client>::new();

        for row in rows {
            let id: Option<i32> = row.try_get("id")?;
            let identity_document_type: Option<IdentityDocumentType> =
                row.try_get("identity_document_type_id")?;
            let identity_document: String = row.try_get("identity_document")?;
            let name: String = row.try_get("name")?;
            let first_surname: String = row.try_get("first_surname")?;
            let second_surname: String = row.try_get("second_surname")?;
            let country: String = row.try_get("country")?;
            let is_deleted: bool = row.try_get("is_deleted")?;
            let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
            let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;

            let search_field: String = format!(
                "{} {} {} {} {} {}",
                id.unwrap_or_default(),
                identity_document,
                name,
                first_surname,
                second_surname,
                country
            );

            let client = Client {
                id,
                identity_document_type,
                identity_document,
                name,
                first_surname,
                second_surname,
                country,
                is_deleted,
                created_at,
                updated_at,
                search_field: search_field.into_boxed_str(),
                ..Default::default()
            };

            result.push(client);
        }
        Ok(result)
    }

    pub async fn get_single(pool: Arc<PgPool>, client_id: i32) -> Result<Client, sqlx::Error> {
        let row = sqlx::query(
            "SELECT 
                clients.id, 
                clients.gender_id, 
                clients.identity_document_type_id, 
                clients.identity_document, 
                clients.identity_document_expedition_date, 
                clients.identity_document_expiration_date, 
                clients.name, 
                clients.first_surname, 
                clients.second_surname, 
                clients.birthdate, 
                clients.address, 
                clients.postal_code, 
                clients.city, 
                clients.province, 
                clients.country, 
                clients.nationality, 
                clients.phone_number, 
                clients.mobile_phone, 
                clients.is_deleted, 
                clients.created_at, 
                clients.updated_at,
                identity_document_types.name as identity_document_type_name,
                genders.name as gender_name 
            FROM clients 
            WHERE clients.id = $1",
        )
        .bind(client_id)
        .fetch_one(pool.as_ref())
        .await?;

        let id: Option<i32> = row.try_get("id")?;
        let gender: Option<Gender> = row.try_get("gender_id")?;
        let identity_document_type: Option<IdentityDocumentType> =
            row.try_get("identity_document_type_id")?;
        let identity_document: String = row.try_get("identity_document")?;
        let identity_document_expedition_date: Option<NaiveDateTime> =
            row.try_get("identity_document_expedition_date")?;
        let identity_document_expiration_date: Option<NaiveDateTime> =
            row.try_get("identity_document_expiration_date")?;
        let name: String = row.try_get("name")?;
        let first_surname: String = row.try_get("first_surname")?;
        let second_surname: String = row.try_get("second_surname")?;
        let birthdate: Option<NaiveDateTime> = row.try_get("birthdate")?;
        let address: String = row.try_get("address")?;
        let postal_code: String = row.try_get("postal_code")?;
        let city: String = row.try_get("city")?;
        let province: String = row.try_get("province")?;
        let country: String = row.try_get("country")?;
        let nationality: String = row.try_get("nationality")?;
        let phone_number: String = row.try_get("phone_number")?;
        let mobile_phone: String = row.try_get("mobile_phone")?;
        let is_deleted: bool = row.try_get("is_deleted")?;
        let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
        let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;

        let birthdate_string: String = if let Some(date) = birthdate {
            format!("{}-{}-{}", date.year(), date.month(), date.day())
        } else {
            String::new()
        };

        let identity_document_expedition_date_string: String =
            if let Some(date) = identity_document_expedition_date {
                format!("{}-{}-{}", date.year(), date.month(), date.day())
            } else {
                String::new()
            };

        let identity_document_expiration_date_string: String =
            if let Some(date) = identity_document_expiration_date {
                format!("{}-{}-{}", date.year(), date.month(), date.day())
            } else {
                String::new()
            };

        let client = Client {
            id,
            gender,
            identity_document_type,
            identity_document,
            identity_document_expedition_date,
            identity_document_expiration_date,
            name,
            first_surname,
            second_surname,
            birthdate,
            address,
            postal_code,
            city,
            province,
            country,
            nationality,
            phone_number,
            mobile_phone,
            is_deleted,
            created_at,
            updated_at,
            birthdate_string,
            identity_document_expedition_date_string,
            identity_document_expiration_date_string,
            ..Default::default()
        };

        Ok(client)
    }

    pub async fn add(pool: Arc<PgPool>, client: Client) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO clients (identity_document_type_id, gender_id, identity_document, identity_document_expedition_date, identity_document_expiration_date, name, first_surname, second_surname, birthdate, address, postal_code, city, province, country, nationality, phone_number, mobile_phone, is_deleted) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)")
            .bind(client.identity_document_type)
            .bind(client.gender)
            .bind(client.identity_document)
            .bind(client.identity_document_expedition_date)
            .bind(client.identity_document_expiration_date)
            .bind(client.name)
            .bind(client.first_surname)
            .bind(client.second_surname)
            .bind(client.birthdate)
            .bind(client.address)
            .bind(client.postal_code)
            .bind(client.city)
            .bind(client.province)
            .bind(client.country)
            .bind(client.nationality)
            .bind(client.phone_number)
            .bind(client.mobile_phone)
            .bind(client.is_deleted)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn edit(pool: Arc<PgPool>, client: Client) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE clients SET identity_document_type_id = $1, identity_document = $2, identity_document_expedition_date = $3, identity_document_expiration_date = $4, name = $5, first_surname = $6, second_surname = $7, birthdate = $8, address = $9, postal_code = $10, city = $11, province = $12, country = $13, nationality = $14, phone_number = $15, mobile_phone = $16, gender_id = $17, is_deleted = $18 WHERE id = $19")
            .bind(client.identity_document_type)
            .bind(client.identity_document)
            .bind(client.identity_document_expedition_date)
            .bind(client.identity_document_expiration_date)
            .bind(client.name)
            .bind(client.first_surname)
            .bind(client.second_surname)
            .bind(client.birthdate)
            .bind(client.address)
            .bind(client.postal_code)
            .bind(client.city)
            .bind(client.province)
            .bind(client.country)
            .bind(client.nationality)
            .bind(client.phone_number)
            .bind(client.mobile_phone)
            .bind(client.gender)
            .bind(client.is_deleted)
            .bind(client.id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }

    pub async fn delete(pool: Arc<PgPool>, client_id: i32) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE clients SET is_deleted = $1 WHERE id = $2")
            .bind(true)
            .bind(client_id)
            .execute(pool.as_ref())
            .await?;

        Ok(())
    }
}
