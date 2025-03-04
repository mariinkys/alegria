use super::screens::bar::TableLocation;

pub fn match_table_location_with_number(tl: TableLocation) -> i32 {
    match tl {
        TableLocation::Bar => 0,
        TableLocation::Resturant => 1,
        TableLocation::Garden => 2,
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum TemporalTicketStatus {
    #[default]
    Pending,
    Printed,
    //Payed,
}

pub fn match_number_with_temporal_ticket_status(n: i32) -> TemporalTicketStatus {
    match n {
        0 => TemporalTicketStatus::Pending,
        1 => TemporalTicketStatus::Printed,
        _ => TemporalTicketStatus::Pending,
    }
}
