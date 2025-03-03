use super::screens::bar::TableLocation;

pub fn match_table_location_with_number(tl: TableLocation) -> i32 {
    match tl {
        TableLocation::Bar => 0,
        TableLocation::Resturant => 1,
        TableLocation::Garden => 2,
    }
}
