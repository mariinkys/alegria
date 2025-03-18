// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use chrono::{Datelike, Local, NaiveDate};
use iced::{Alignment, Element, Length, Pixels, Task, widget};
use iced_aw::{
    DatePicker,
    date_picker::{self, Date},
};
use sqlx::PgPool;

use crate::{
    alegria::{
        action::AlegriaAction,
        core::models::{reservation::Reservation, room::Room},
        utils::{check_date_format, parse_date_to_naive_datetime},
    },
    fl,
};

#[derive(Debug, Clone, PartialEq)]
enum ReservationsScreen {
    Home,
}

#[derive(Debug, Clone)]
struct DateFiltersState {
    initial_date: NaiveDate,
    show_initial_date_picker: bool,
    initial_date_string: String,
    last_date: NaiveDate,
    show_last_date_picker: bool,
    last_date_string: String,
}

impl Default for DateFiltersState {
    fn default() -> Self {
        let initial_date = Local::now().date_naive();

        let last_date = Local::now()
            .date_naive()
            .checked_add_days(chrono::Days::new(14))
            .unwrap_or(Local::now().date_naive());
        Self {
            initial_date,
            show_initial_date_picker: false,
            initial_date_string: initial_date.to_string(),
            last_date,
            show_last_date_picker: false,
            last_date_string: last_date.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReservationDateInputFields {
    FilterInitialDate,
    FilterLastDate,
}

#[derive(Debug, Clone)]
pub enum ReservationTextInputFields {
    FilterInitialDate,
    FilterLastDate,
}

pub struct Reservations {
    /// Database of the application
    pub database: Option<Arc<PgPool>>,
    /// Determines which is the current view of the subscreen
    current_screen: ReservationsScreen,
    /// Holds the state of all the reservations (of the given time/period)
    reservations: Vec<Reservation>,
    /// Holds the state of all the rooms (needed to create the kinda grid/calendar view)
    rooms: Vec<Room>,
    /// Holds the state of the date filters that control which reservations are being shown
    date_filters: DateFiltersState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back

    InitPage, // Intended to be called from Hotel when first opening the page, asks for the necessary data and executes the appropiate callbacks

    FetchReservations,                 // Fetches all the reservations
    SetReservations(Vec<Reservation>), // Sets the reservations on the app state

    //FetchRooms, // Fetches all the rooms
    SetRooms(Vec<Room>), // Sets the rooms on the app state

    TextInputUpdate(String, ReservationTextInputFields), // Callback when using the text inputs of the reservations page
    ShowDatePicker(ReservationDateInputFields),          // Asks to open the requesed date picker
    CancelDateOperation,                                 // Cancels the datepicker changes
    UpdateDateField(date_picker::Date, ReservationDateInputFields), // Callback after submiting a new date via datepicker
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum ReservationsInstruction {
    Back, // Asks the parent (app.rs) to go back
}

impl Reservations {
    /// Initializes the screen
    pub fn init() -> Self {
        Self {
            database: None,
            current_screen: ReservationsScreen::Home,
            reservations: Vec::new(),
            rooms: Vec::new(),
            date_filters: DateFiltersState::default(),
        }
    }

    /// Cleans the state of the screen preserving the database
    /// intended to be called when switching to another screen in order to save memory.
    pub fn clean_state(database: Option<Arc<PgPool>>) -> Self {
        Self {
            database,
            current_screen: ReservationsScreen::Home,
            reservations: Vec::new(),
            rooms: Vec::new(),
            date_filters: DateFiltersState::default(),
        }
    }

    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<ReservationsInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            Message::Back => action.add_instruction(ReservationsInstruction::Back),

            // Intended to be called from Hotel when first opening the page, asks for the necessary data and executes the appropiate callbacks
            Message::InitPage => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        Reservation::get_all(
                            pool.clone(),
                            self.date_filters.initial_date,
                            self.date_filters.last_date,
                        ),
                        |res| match res {
                            Ok(res) => Message::SetReservations(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetReservations(Vec::new())
                            }
                        },
                    ));

                    action.add_task(Task::perform(
                        Room::get_all(pool.clone()),
                        |res| match res {
                            Ok(res) => Message::SetRooms(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetRooms(Vec::new())
                            }
                        },
                    ));
                }
            }

            // Fetches all the reservations
            Message::FetchReservations => {
                if check_date_format(&self.date_filters.initial_date_string)
                    && check_date_format(&self.date_filters.last_date_string)
                {
                    let intial_date =
                        parse_date_to_naive_datetime(&self.date_filters.initial_date_string);
                    let last_date =
                        parse_date_to_naive_datetime(&self.date_filters.last_date_string);

                    if intial_date.is_some() && last_date.is_some() {
                        self.date_filters.initial_date = intial_date.unwrap().date();
                        self.date_filters.last_date = last_date.unwrap().date();

                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                Reservation::get_all(
                                    pool.clone(),
                                    self.date_filters.initial_date,
                                    self.date_filters.last_date,
                                ),
                                |res| match res {
                                    Ok(res) => Message::SetReservations(res),
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::SetReservations(Vec::new())
                                    }
                                },
                            ));
                        }
                    };
                } else {
                    eprintln!("Can't parse dates"); // TODO: Toast
                }
            }
            // Sets the reservations on the app state
            Message::SetReservations(res) => {
                self.reservations = res;
            }

            // Sets the rooms on the app state
            Message::SetRooms(res) => {
                self.rooms = res;
            }

            // Callback when using the text inputs of the reservations page
            Message::TextInputUpdate(new_value, field) => match field {
                ReservationTextInputFields::FilterInitialDate => {
                    self.date_filters.initial_date_string = new_value
                }
                ReservationTextInputFields::FilterLastDate => {
                    self.date_filters.last_date_string = new_value
                }
            },
            // Asks to open the requesed date picker
            Message::ShowDatePicker(field) => match field {
                ReservationDateInputFields::FilterInitialDate => {
                    self.date_filters.show_initial_date_picker = true;
                }
                ReservationDateInputFields::FilterLastDate => {
                    self.date_filters.show_last_date_picker = true;
                }
            },
            // Cancels the datepicker changes
            Message::CancelDateOperation => {
                self.date_filters.show_initial_date_picker = false;
                self.date_filters.show_last_date_picker = false;
            }
            // Callback after submiting a new date via datepicker
            Message::UpdateDateField(iced_aw_date, field) => {
                let new_date = NaiveDate::from_ymd_opt(
                    iced_aw_date.year,
                    iced_aw_date.month,
                    iced_aw_date.day,
                );

                match new_date {
                    Some(date) => {
                        match field {
                            ReservationDateInputFields::FilterInitialDate => {
                                self.date_filters.initial_date = date;
                                self.date_filters.initial_date_string =
                                    format!("{}-{}-{}", date.year(), date.month(), date.day())
                            }
                            ReservationDateInputFields::FilterLastDate => {
                                self.date_filters.last_date = date;
                                self.date_filters.last_date_string =
                                    format!("{}-{}-{}", date.year(), date.month(), date.day())
                            }
                        }
                        self.date_filters.show_initial_date_picker = false;
                        self.date_filters.show_last_date_picker = false;
                    }
                    None => {
                        eprintln!("Could not parse new date");
                        match field {
                            ReservationDateInputFields::FilterInitialDate => {
                                self.date_filters.initial_date = Local::now().date_naive();
                                self.date_filters.initial_date_string =
                                    self.date_filters.initial_date.to_string();
                            }
                            ReservationDateInputFields::FilterLastDate => {
                                self.date_filters.last_date = Local::now()
                                    .date_naive()
                                    .checked_add_days(chrono::Days::new(14))
                                    .unwrap_or(Local::now().date_naive());
                                self.date_filters.last_date_string =
                                    self.date_filters.last_date.to_string();
                            }
                        }
                    }
                }
            }
        };

        action
    }

    const GLOBAL_SPACING: f32 = 6.;
    const GLOBAL_BUTTON_HEIGHT: f32 = 60.;

    /// Returns the view of the subscreen
    pub fn view(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        // HEADER
        let header_row = self.view_header_row();
        let content = match self.current_screen {
            ReservationsScreen::Home => self.view_reservations_calendar(),
        };

        widget::Column::new()
            .push(header_row)
            .push(content)
            .spacing(spacing)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    //
    //  VIEW COMPOSING
    //

    const TITLE_TEXT_SIZE: f32 = 25.0;
    const TEXT_SIZE: f32 = 18.0;

    /// Returns the view of the header row of the subscreen
    fn view_header_row(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        let back_button = widget::Button::new(
            widget::Text::new(fl!("back"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .on_press(Message::Back)
        .height(button_height);

        widget::Row::new()
            .push(back_button)
            .push(
                widget::Text::new(fl!("reservations"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .align_y(Alignment::Center),
            )
            .push(widget::Space::new(Length::Fill, Length::Shrink))
            .push(self.view_date_pickers_row())
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .spacing(spacing)
            .into()
    }

    /// Returns the row of date pickers (for the heaeder row)
    fn view_date_pickers_row(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        // Initial Date
        let initial_date_label =
            widget::Text::new(format!("{} (yyyy-mm-dd)", fl!("initial-date"))).width(Length::Fill);
        let initial_date_iced_aw_date = Date {
            year: self.date_filters.initial_date.year(),
            month: self.date_filters.initial_date.month(),
            day: self.date_filters.initial_date.day(),
        };
        let initial_date_picker = DatePicker::new(
            self.date_filters.show_initial_date_picker,
            initial_date_iced_aw_date,
            widget::Button::new(widget::Text::new(fl!("edit"))).on_press(Message::ShowDatePicker(
                ReservationDateInputFields::FilterInitialDate,
            )),
            Message::CancelDateOperation,
            |date| Message::UpdateDateField(date, ReservationDateInputFields::FilterInitialDate),
        );
        let initial_date_input = widget::TextInput::new(
            fl!("initial-date").as_str(),
            &self.date_filters.initial_date_string,
        )
        .on_input(|c| Message::TextInputUpdate(c, ReservationTextInputFields::FilterInitialDate))
        .size(Pixels::from(Self::TEXT_SIZE))
        .width(Length::Fill);

        let initial_date_input_row = widget::Row::new()
            .push(initial_date_input)
            .push(initial_date_picker)
            .align_y(Alignment::Center)
            .spacing(1.);
        let initial_date_input_column = widget::Column::new()
            .push(initial_date_label)
            .push(initial_date_input_row)
            .width(Length::Fill)
            .spacing(1.);

        // Last Date
        let last_date_label =
            widget::Text::new(format!("{} (yyyy-mm-dd)", fl!("last-date"))).width(Length::Fill);
        let last_date_iced_aw_date = Date {
            year: self.date_filters.last_date.year(),
            month: self.date_filters.last_date.month(),
            day: self.date_filters.last_date.day(),
        };
        let last_date_picker = DatePicker::new(
            self.date_filters.show_last_date_picker,
            last_date_iced_aw_date,
            widget::Button::new(widget::Text::new(fl!("edit"))).on_press(Message::ShowDatePicker(
                ReservationDateInputFields::FilterLastDate,
            )),
            Message::CancelDateOperation,
            |date| Message::UpdateDateField(date, ReservationDateInputFields::FilterLastDate),
        );
        let last_date_input = widget::TextInput::new(
            fl!("last-date").as_str(),
            &self.date_filters.last_date_string,
        )
        .on_input(|c| Message::TextInputUpdate(c, ReservationTextInputFields::FilterLastDate))
        .size(Pixels::from(Self::TEXT_SIZE))
        .width(Length::Fill);

        let last_date_input_row = widget::Row::new()
            .push(last_date_input)
            .push(last_date_picker)
            .align_y(Alignment::Center)
            .spacing(1.);
        let last_date_input_column = widget::Column::new()
            .push(last_date_label)
            .push(last_date_input_row)
            .width(Length::Fill)
            .spacing(1.);

        // Submit button
        let submit_button = widget::Button::new(
            widget::Text::new(fl!("refresh"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .size(Pixels::from(Self::TEXT_SIZE)),
        )
        .on_press(Message::FetchReservations)
        .height(button_height)
        .width(Length::Shrink);

        widget::Row::new()
            .push(initial_date_input_column)
            .push(last_date_input_column)
            .push(submit_button)
            .align_y(Alignment::Center)
            .spacing(spacing)
            .into()
    }

    /// Returns the view of the header row of the subscreen
    fn view_reservations_calendar(&self) -> Element<Message> {
        let cell_width = Length::Fill; // If I put a fixed width here I also have to put it on the Header Row or everything breaks
        let cell_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        // header row with days
        let mut header_row = widget::Row::new();

        // top left empty cell
        header_row = header_row
            .push(
                widget::Text::new("")
                    .size(16)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(cell_width)
                    .height(cell_height),
            )
            .width(Length::Fill)
            .spacing(spacing);

        // add each day of range as a header
        let mut current_date = self.date_filters.initial_date;
        while current_date <= self.date_filters.last_date {
            header_row = header_row.push(
                widget::Text::new(format!("{}/{}", current_date.day(), current_date.month()))
                    .size(16)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(cell_width)
                    .height(cell_height),
            );
            current_date += chrono::Duration::days(1);
        }

        // final calendar view
        let mut calendar_view = widget::Column::new().push(header_row).spacing(spacing);

        for room in &self.rooms {
            // each room is a row
            let mut row = widget::Row::new();
            row = row.push(
                widget::Text::new(&room.name)
                    .size(16)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(cell_width)
                    .height(cell_height),
            );

            // loop through each day in the range and check for reservations
            let mut current_date = self.date_filters.initial_date;
            while current_date <= self.date_filters.last_date {
                let mut cell_content = widget::Text::new("N")
                    .size(16)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(cell_width)
                    .height(cell_height);

                for reservation in &self.reservations {
                    // check if the current room is part of the reservation and if the date falls within the reservation period
                    if reservation.rooms.iter().any(|r| r.id == room.id)
                        && reservation.entry_date.unwrap_or_default().date() <= current_date
                        && reservation.departure_date.unwrap_or_default().date() > current_date
                    {
                        // &reservation.client_name
                        cell_content = widget::Text::new(format!(
                            "S:{}/{}",
                            reservation.entry_date.unwrap_or_default().date().day(),
                            reservation.departure_date.unwrap_or_default().date().day()
                        ))
                        .size(16)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .width(cell_width)
                        .height(cell_height);
                        break; // each room can only have one reservation per day
                    }
                }

                row = row.push(cell_content);
                current_date += chrono::Duration::days(1);
            }

            calendar_view = calendar_view.push(row);
        }

        widget::Container::new(calendar_view).into()
    }

    //
    //  END OF VIEW COMPOSING
    //
}
