// SPDX-License-Identifier: GPL-3.0-only

mod add;
mod edit;

use std::sync::Arc;

use add::AddReservationPage;
use chrono::{Datelike, Local, NaiveDate};
use iced::{
    Alignment, Element, Length, Padding, Pixels, Task,
    widget::{self},
};
use iced_aw::{
    DatePicker,
    date_picker::{self, Date},
};
use sqlx::PgPool;

use crate::{
    alegria::{
        action::AlegriaAction,
        core::models::{reservation::Reservation, room::Room, sold_room::SoldRoom},
        utils::{check_date_format, error_toast, parse_date_to_naive_datetime},
        widgets::toast::{self, Toast},
    },
    fl,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ReservationsScreen {
    Home,
    Add,
    //Edit,
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

#[derive(Debug, Clone)]
pub enum ReservationDirectionAction {
    Back,
    Forward,
}

pub struct Reservations {
    /// Database of the application
    pub database: Option<Arc<PgPool>>,
    /// Page Toasts
    toasts: Vec<Toast>,
    /// Determines which is the current view of the subscreen
    current_screen: ReservationsScreen,
    /// Holds the state of all the reservations (of the given time/period)
    reservations: Arc<Vec<Reservation>>,
    /// Holds the state of all the rooms (needed to create the kinda grid/calendar view)
    rooms: Arc<Vec<Room>>,
    /// Holds the state of the date filters that control which reservations are being shown
    date_filters: DateFiltersState,
    /// Add SubScreen of the reservation page
    add_reservations: AddReservationPage,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back

    AddToast(Toast),   // Adds the given toast to the state to be shown on screen
    CloseToast(usize), // Callback after clicking the close toast button

    InitPage, // Intended to be called from Hotel when first opening the page, asks for the necessary data and executes the appropiate callbacks
    OpenAddReservationForm(NaiveDate, Room), // Changes the current screen to add reservation and sets the needed variables for creating a new reservation

    FetchReservations,                 // Fetches all the reservations
    SetReservations(Vec<Reservation>), // Sets the reservations on the app state

    SetRooms(Vec<Room>), // Sets the rooms on the app state

    TextInputUpdate(String, ReservationTextInputFields), // Callback when using the text inputs of the reservations page
    ShowDatePicker(ReservationDateInputFields),          // Asks to open the requesed date picker
    CancelDateOperation,                                 // Cancels the datepicker changes
    UpdateDateField(date_picker::Date, ReservationDateInputFields), // Callback after submiting a new date via datepicker
    DirectionActionInput(ReservationDirectionAction), // Callback after clicking one of the two arrows to go one day back/forward

    AddReservationPage(self::add::Message), // Messages of the add reservations page
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum ReservationsInstruction {
    Back, // Asks the parent (app.rs) to go back
}

impl Default for Reservations {
    fn default() -> Self {
        Self {
            database: None,
            toasts: Vec::new(),
            current_screen: ReservationsScreen::Home,
            reservations: Arc::default(),
            rooms: Arc::default(),
            date_filters: DateFiltersState::default(),
            add_reservations: AddReservationPage::default(),
        }
    }
}

impl Reservations {
    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<ReservationsInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            Message::Back => match self.current_screen {
                ReservationsScreen::Home => action.add_instruction(ReservationsInstruction::Back),
                ReservationsScreen::Add => {
                    self.add_reservations = AddReservationPage::default();
                    self.current_screen = ReservationsScreen::Home;
                    return self.update(Message::FetchReservations);
                } //ReservationsScreen::Edit => todo!(),
            },

            // Adds the given toast to the state to be shown on screen
            Message::AddToast(toast) => {
                self.toasts.push(toast);
            }
            // Callback after clicking the close toast button
            Message::CloseToast(index) => {
                self.toasts.remove(index);
            }

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

                    self.current_screen = ReservationsScreen::Home;
                }
            }
            // Changes the current screen of the reservations page
            Message::OpenAddReservationForm(reservation_initial_date, clicked_room) => {
                let mut reservation = Reservation {
                    entry_date: Some(reservation_initial_date.and_hms_opt(0, 0, 0).unwrap()),
                    departure_date: Some(
                        reservation_initial_date
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .checked_add_days(chrono::Days::new(1))
                            .unwrap(),
                    ),
                    ..Default::default()
                };

                // we only add the clicked room to the reservation if it's available on the selected dates
                let can_add_room = !self.reservations.iter().any(|r| {
                    r.rooms.iter().any(|r| r.room_id == clicked_room.id)
                        && r.entry_date.unwrap() < reservation.departure_date.unwrap()
                        && r.departure_date.unwrap() > reservation.entry_date.unwrap()
                });
                if can_add_room {
                    reservation.rooms.push(SoldRoom {
                        id: None,
                        room_id: clicked_room.id,
                        guests: Vec::new(),
                        price: clicked_room.default_room_price,
                        invoices: Vec::new(),
                    });
                }

                self.add_reservations = AddReservationPage::open_reservation(
                    self.database.clone(),
                    reservation,
                    self.rooms.clone(),
                    self.reservations.clone(),
                );
                self.current_screen = ReservationsScreen::Add;
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
                        #[allow(clippy::collapsible_if)]
                        if intial_date.unwrap().date() < last_date.unwrap().date() {
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
                        }
                    };
                } else {
                    self.toasts
                        .push(error_toast(String::from("Can't parse dates")));
                    eprintln!("Can't parse dates");
                }
            }
            // Sets the reservations on the app state
            Message::SetReservations(res) => {
                self.reservations = Arc::from(res);
            }

            // Sets the rooms on the app state
            Message::SetRooms(res) => {
                self.rooms = Arc::from(res);
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
                                if date < self.date_filters.last_date {
                                    self.date_filters.initial_date = date;
                                    self.date_filters.initial_date_string =
                                        format!("{}-{}-{}", date.year(), date.month(), date.day());
                                    return self.update(Message::FetchReservations);
                                }
                            }
                            ReservationDateInputFields::FilterLastDate => {
                                if date > self.date_filters.initial_date {
                                    self.date_filters.last_date = date;
                                    self.date_filters.last_date_string =
                                        format!("{}-{}-{}", date.year(), date.month(), date.day());
                                    return self.update(Message::FetchReservations);
                                }
                            }
                        }

                        // close all date pickers
                        self.date_filters.show_initial_date_picker = false;
                        self.date_filters.show_last_date_picker = false;
                    }
                    None => {
                        self.toasts
                            .push(error_toast(String::from("Could not parse new date")));
                        eprintln!("Could not parse new date");
                        return self.update(Message::FetchReservations);
                    }
                }
            }
            // Callback after clicking one of the two arrows to go one day back/forward
            Message::DirectionActionInput(action) => match action {
                ReservationDirectionAction::Back => {
                    if let Some(new_in_date) = self
                        .date_filters
                        .initial_date
                        .checked_sub_days(chrono::Days::new(1))
                    {
                        if let Some(new_l_date) = self
                            .date_filters
                            .last_date
                            .checked_sub_days(chrono::Days::new(1))
                        {
                            self.date_filters.initial_date = new_in_date;
                            self.date_filters.initial_date_string = format!(
                                "{}-{}-{}",
                                new_in_date.year(),
                                new_in_date.month(),
                                new_in_date.day()
                            );
                            self.date_filters.last_date = new_l_date;
                            self.date_filters.last_date_string = format!(
                                "{}-{}-{}",
                                new_l_date.year(),
                                new_l_date.month(),
                                new_l_date.day()
                            );
                            return self.update(Message::FetchReservations);
                        }
                    }
                }
                ReservationDirectionAction::Forward => {
                    if let Some(new_in_date) = self
                        .date_filters
                        .initial_date
                        .checked_add_days(chrono::Days::new(1))
                    {
                        if let Some(new_l_date) = self
                            .date_filters
                            .last_date
                            .checked_add_days(chrono::Days::new(1))
                        {
                            self.date_filters.initial_date = new_in_date;
                            self.date_filters.initial_date_string = format!(
                                "{}-{}-{}",
                                new_in_date.year(),
                                new_in_date.month(),
                                new_in_date.day()
                            );
                            self.date_filters.last_date = new_l_date;
                            self.date_filters.last_date_string = format!(
                                "{}-{}-{}",
                                new_l_date.year(),
                                new_l_date.month(),
                                new_l_date.day()
                            );
                            return self.update(Message::FetchReservations);
                        }
                    }
                }
            },

            // Messages of the add reservations page
            Message::AddReservationPage(message) => {
                let add_reservation_action = self.add_reservations.update(message);

                let add_reservation_tasks: Vec<Task<Message>> = add_reservation_action
                    .tasks
                    .into_iter()
                    .map(|task| task.map(Message::AddReservationPage))
                    .collect();
                action.tasks.extend(add_reservation_tasks);

                for instructions in add_reservation_action.instructions {
                    match instructions {
                        add::AddReservationsInstruction::Back => {
                            return self.update(Message::Back);
                        }
                        add::AddReservationsInstruction::TryAddReservation(reservation) => {
                            if let Some(pool) = &self.database {
                                action.add_task(Task::perform(
                                    Reservation::add(pool.clone(), reservation.clone()),
                                    |res| match res {
                                        Ok(_) => Message::Back,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::AddToast(error_toast(err.to_string()))
                                        }
                                    },
                                ));
                            }
                        }
                        add::AddReservationsInstruction::ShowToast(toast) => {
                            self.toasts.push(toast);
                        }
                    }
                }
            }
        };

        action
    }

    const GLOBAL_SPACING: f32 = 6.;
    const GLOBAL_BUTTON_HEIGHT: f32 = 60.;
    const TITLE_TEXT_SIZE: f32 = 25.0;
    const TEXT_SIZE: f32 = 18.0;

    /// Returns the view of the subscreen
    pub fn view(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        // HEADER
        let header_row = match self.current_screen {
            ReservationsScreen::Home => self.view_header_row(),
            ReservationsScreen::Add => self
                .add_reservations
                .view_header_row()
                .map(Message::AddReservationPage),
        };
        let content = match self.current_screen {
            ReservationsScreen::Home => self.view_reservations_calendar(),
            ReservationsScreen::Add => self
                .add_reservations
                .view()
                .map(Message::AddReservationPage),
            //ReservationsScreen::Edit => todo!(),
        };

        toast::Manager::new(
            widget::Column::new()
                .push(header_row)
                .push(content)
                .spacing(spacing)
                .height(Length::Fill)
                .width(Length::Fill),
            &self.toasts,
            Message::CloseToast,
        )
        .into()
    }

    //
    //  VIEW COMPOSING
    //

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
        .on_submit(Message::FetchReservations)
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
        .on_submit(Message::FetchReservations)
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

        widget::Row::new()
            .push(initial_date_input_column)
            .push(last_date_input_column)
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

        // top left action buttons
        header_row = header_row
            .push(
                widget::Row::new()
                    .push(
                        widget::Button::new(
                            widget::Text::new("<")
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center)
                                .height(Length::Fill)
                                .width(Length::Fill),
                        )
                        .style(widget::button::secondary)
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .on_press(Message::DirectionActionInput(
                            ReservationDirectionAction::Back,
                        )),
                    )
                    .push(
                        widget::Button::new(
                            widget::Text::new(">")
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center)
                                .height(Length::Fill)
                                .width(Length::Fill),
                        )
                        .style(widget::button::secondary)
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .on_press(Message::DirectionActionInput(
                            ReservationDirectionAction::Forward,
                        )),
                    )
                    .align_y(Alignment::Center)
                    .spacing(spacing)
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

        for room in &*self.rooms {
            // each room is a row
            let mut row = widget::Row::new().spacing(spacing);
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
                let mut cell_content = widget::Container::new(
                    widget::Button::new("")
                        .on_press(Message::OpenAddReservationForm(current_date, room.clone()))
                        .style(widget::button::secondary)
                        .width(cell_width)
                        .height(cell_height),
                );

                for reservation in &*self.reservations {
                    // check if the current room is part of the reservation and if the date falls within the reservation period
                    if reservation.rooms.iter().any(|r| r.room_id == room.id)
                        && reservation.entry_date.unwrap_or_default().date() <= current_date
                        // departure date does not have an equal because we can book a room the day someone departs
                        && reservation.departure_date.unwrap_or_default().date() > current_date
                    {
                        match reservation.occupied {
                            true => {
                                cell_content = widget::Container::new(widget::Tooltip::new(
                                    widget::Button::new("")
                                        .style(widget::button::success)
                                        .width(cell_width)
                                        .height(cell_height),
                                    widget::container(reservation.client_name.as_str())
                                        .style(widget::container::rounded_box)
                                        .padding(Padding::new(3.)),
                                    widget::tooltip::Position::FollowCursor,
                                ));
                            }
                            false => {
                                cell_content = widget::Container::new(widget::Tooltip::new(
                                    widget::Button::new("")
                                        .style(widget::button::danger)
                                        .width(cell_width)
                                        .height(cell_height),
                                    widget::container(reservation.client_name.as_str())
                                        .style(widget::container::rounded_box)
                                        .padding(Padding::new(3.)),
                                    widget::tooltip::Position::FollowCursor,
                                ));
                            }
                        }
                        break; // each room can only have one reservation per day
                    }
                }

                row = row.push(cell_content);
                current_date += chrono::Duration::days(1);
            }

            calendar_view = calendar_view.push(row);
        }

        widget::Container::new(widget::Scrollable::new(calendar_view)).into()
    }
}
