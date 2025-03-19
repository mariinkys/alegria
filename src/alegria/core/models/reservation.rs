// SPDX-License-Identifier: GPL-3.0-only

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;

use super::sold_room::SoldRoom;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reservation {
    pub id: Option<i32>,
    pub client_id: Option<i32>,
    pub rooms: Vec<SoldRoom>,
    pub entry_date: Option<NaiveDateTime>,
    pub departure_date: Option<NaiveDateTime>,
    pub occupied: bool,
    pub is_deleted: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,

    // Not in the db
    pub client_name: String, // Helps us JOIN and return the name of the selected client
}

#[allow(clippy::derivable_impls)]
impl Default for Reservation {
    fn default() -> Self {
        Self {
            id: None,
            client_id: None,
            rooms: Vec::new(),
            entry_date: None,
            departure_date: None,
            occupied: false,
            is_deleted: false,
            created_at: None,
            updated_at: None,
            client_name: String::new(),
        }
    }
}

impl Reservation {
    /// Retrieves all the reservations, but only the fields needed for the grid and main page of the reservation
    pub async fn get_all(
        pool: Arc<PgPool>,
        initial_date: NaiveDate,
        last_date: NaiveDate,
    ) -> Result<Vec<Reservation>, sqlx::Error> {
        // convert NaiveDate to NaiveDateTime (taking into account start/end of the day)
        let initial_datetime = initial_date.and_hms_opt(0, 0, 0).unwrap();
        let last_datetime = last_date.and_hms_opt(23, 59, 59).unwrap();

        let rows = sqlx::query(
            "SELECT 
                reservations.id, 
                reservations.client_id, 
                reservations.entry_date, 
                reservations.departure_date, 
                reservations.occupied, 
                reservations.is_deleted, 
                reservations.created_at, 
                reservations.updated_at,
                clients.name as client_name,
                clients.first_surname as client_first_surname,
                clients.second_surname as client_second_surname
            FROM reservations 
            LEFT JOIN clients ON reservations.client_id = clients.id 
            WHERE reservations.is_deleted = $1
            AND (
                (reservations.entry_date BETWEEN $2 AND $3) 
                OR (reservations.departure_date BETWEEN $2 AND $3) 
                OR (reservations.entry_date <= $2 AND reservations.departure_date >= $3)
            ) 
            ORDER BY reservations.id DESC",
        )
        .bind(false)
        .bind(initial_datetime)
        .bind(last_datetime)
        .fetch_all(pool.as_ref())
        .await?;

        let mut result = Vec::<Reservation>::new();

        for row in rows {
            let id: Option<i32> = row.try_get("id")?;
            let client_id: Option<i32> = row.try_get("client_id")?;
            let entry_date: Option<NaiveDateTime> = row.try_get("entry_date")?;
            let departure_date: Option<NaiveDateTime> = row.try_get("departure_date")?;
            let occupied: bool = row.try_get("occupied")?;
            let is_deleted: bool = row.try_get("is_deleted")?;
            let created_at: Option<NaiveDateTime> = row.try_get("created_at")?;
            let updated_at: Option<NaiveDateTime> = row.try_get("updated_at")?;
            let client_name: String = row.try_get("client_name").unwrap_or_default();
            let client_first_surname: String =
                row.try_get("client_first_surname").unwrap_or_default();
            let client_second_surname: String =
                row.try_get("client_second_surname").unwrap_or_default();

            let client_name = format!(
                "{} {} {}",
                client_name, client_first_surname, client_second_surname
            );

            // get rooms for this reservation
            let rooms = if let Some(reservation_id) = id {
                let room_rows = sqlx::query(
                    "SELECT 
                    sr.id, 
                    sr.room_id, 
                    sr.price
                FROM sold_rooms sr
                JOIN reservation_sold_rooms rsr ON sr.id = rsr.sold_room_id
                WHERE rsr.reservation_id = $1",
                )
                .bind(reservation_id)
                .fetch_all(pool.as_ref())
                .await?;

                let mut rooms = Vec::new();

                for room_row in room_rows {
                    let sold_room = SoldRoom {
                        id: room_row.try_get("id")?,
                        room_id: room_row.try_get("room_id")?,
                        price: room_row.try_get("price")?,
                        guests: Vec::new(),
                        invoices: Vec::new(),
                    };
                    rooms.push(sold_room);
                }
                rooms
            } else {
                Vec::new()
            };

            let reservation = Reservation {
                id,
                client_id,
                rooms,
                entry_date,
                departure_date,
                occupied,
                is_deleted,
                created_at,
                updated_at,
                client_name,
            };

            result.push(reservation);
        }

        Ok(result)
    }
}
