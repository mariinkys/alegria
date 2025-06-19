// SPDX-License-Identifier: GPL-3.0-only

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum TemporalTicketStatus {
    #[default]
    Pending,
    Printed,
    //Paid,
}

pub fn match_number_with_temporal_ticket_status(n: i32) -> TemporalTicketStatus {
    match n {
        0 => TemporalTicketStatus::Pending,
        1 => TemporalTicketStatus::Printed,
        _ => TemporalTicketStatus::Pending,
    }
}
