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

    /// Adds the given reservation with it's rooms to the database
    pub async fn add(pool: Arc<PgPool>, reservation: Reservation) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        // check if rooms are available for the given date range
        if let (Some(entry_date), Some(departure_date)) =
            (reservation.entry_date, reservation.departure_date)
        {
            if entry_date >= departure_date {
                return Err(sqlx::Error::Protocol(
                    "entry date must be before departure date".to_string(),
                ));
            }

            for sold_room in &reservation.rooms {
                let overlapping_count = sqlx::query(
                    "SELECT COUNT(*) FROM reservations r
                        JOIN reservation_sold_rooms rsr ON r.id = rsr.reservation_id
                        JOIN sold_rooms sr ON rsr.sold_room_id = sr.id
                        WHERE sr.room_id = $1
                        AND r.is_deleted = false
                        AND r.entry_date < $3  -- existing entry is before new departure
                        AND r.departure_date > $2  -- existing departure is after new entry
                        AND NOT (r.departure_date = $2)  -- allow booking when existing departure equals new entry
                    "
                )
                .bind(sold_room.room_id)
                .bind(entry_date)
                .bind(departure_date)
                .fetch_one(&mut *tx)
                .await?;

                let count: i64 = overlapping_count.get(0);
                if count > 0 {
                    return Err(sqlx::Error::Protocol(format!(
                        "Room {:?} is already reserved for the selected date range",
                        sold_room.room_id
                    )));
                }
            }
        } else {
            return Err(sqlx::Error::Protocol(
                "entry and departure dates are required".to_string(),
            ));
        }

        // Insert the reservation
        let reservation_id = sqlx::query("INSERT INTO reservations (client_id, entry_date, departure_date, occupied, is_deleted, created_at, updated_at) 
            VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) 
            RETURNING id",
        )
        .bind(reservation.client_id)
        .bind(reservation.entry_date)
        .bind(reservation.departure_date)
        .bind(reservation.occupied)
        .bind(reservation.is_deleted)
        .fetch_one(&mut *tx)
        .await?
        .get::<i32, _>(0);

        // create all sold_rooms for the reservation
        for sold_room in &reservation.rooms {
            // create sold_room
            let sold_room_id =
                sqlx::query("INSERT INTO sold_rooms (room_id, price) VALUES ($1, $2) RETURNING id")
                    .bind(sold_room.room_id)
                    .bind(sold_room.price)
                    .fetch_one(&mut *tx)
                    .await?
                    .get::<i32, _>(0);

            // insert association in reservation_sold_rooms
            sqlx::query(
                "INSERT INTO reservation_sold_rooms (reservation_id, sold_room_id) VALUES ($1, $2)",
            )
            .bind(reservation_id)
            .bind(sold_room_id)
            .execute(&mut *tx)
            .await?;
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }
}
