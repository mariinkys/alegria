// SPDX-License-Identifier: GPL-3.0-only

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

/// Checks if a date (string) is on a valid format of yyyy-(m)m-(d)d
pub fn check_date_format(date: &str) -> bool {
    let parts: Vec<&str> = date.split('-').collect();

    if parts.len() != 3 {
        return false;
    }

    // Check if year is 4 digits
    if parts[0].len() != 4 || !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Check if month is 1 or 2 digits and between 1-12
    if parts[1].is_empty() || !parts[1].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let month = parts[1].parse::<u32>().unwrap_or(0);
    if !(1..=12).contains(&month) {
        return false;
    }

    // Check if day is 1-2 digits and between 1-31
    if parts[2].is_empty() || !parts[2].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let day = parts[2].parse::<u32>().unwrap_or(0);
    if !(1..=31).contains(&day) {
        return false;
    }

    true
}

/// Gets a date with format yyyy-(m)m-(d)d and returns a naive date time
pub fn parse_date_to_naive_datetime(date: &str) -> Option<NaiveDateTime> {
    // Split the date string by '-'
    let parts: Vec<&str> = date.split('-').collect();

    // Check if we have exactly 3 parts
    if parts.len() != 3 {
        return None;
    }

    // Parse the year component
    let year = match parts[0].parse::<i32>() {
        Ok(y) => y,
        Err(_) => return None,
    };

    // Parse the month component
    let month = match parts[1].parse::<u32>() {
        Ok(m) if (1..=12).contains(&m) => m,
        _ => return None,
    };

    // Parse the day component
    let day = match parts[2].parse::<u32>() {
        Ok(d) if (1..=31).contains(&d) => d,
        _ => return None,
    };

    // Try to create a NaiveDate
    NaiveDate::from_ymd_opt(year, month, day).map(|date| NaiveDateTime::new(date, NaiveTime::MIN))
}
