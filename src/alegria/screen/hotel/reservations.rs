// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use chrono::{Datelike, Local, NaiveDate};
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::time::Instant;
use iced::widget::{
    Column, Row, Space, Tooltip, button, column, focus_next, focus_previous, row, scrollable, text,
    text_input, tooltip,
};
use iced::{Alignment, Length, Subscription, event};
use iced::{Task, widget::container};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::reservation::Reservation;
use crate::alegria::core::models::room::Room;
use crate::alegria::utils::date::{check_date_format, parse_date_to_naive_datetime};
use crate::alegria::utils::styling::{
    GLOBAL_BUTTON_HEIGHT, GLOBAL_SPACING, TEXT_SIZE, TITLE_TEXT_SIZE,
};

use crate::{alegria::widgets::toast::Toast, fl};

mod add;
mod edit;

pub struct Reservations {
    state: State,
}

enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

enum SubScreen {
    List {
        date_filters: DateFilters,
        reservations: Vec<Reservation>,
        rooms: Vec<Room>,
    },
    Add(add::AddReservation),
    Edit(edit::EditReservation),
}

#[derive(Debug, Clone)]
pub struct DateFilters {
    initial_date: NaiveDate,
    initial_date_string: String,
    last_date: NaiveDate,
    last_date_string: String,
}

impl Default for DateFilters {
    fn default() -> Self {
        let initial_date = Local::now().date_naive();
        let last_date = Local::now()
            .checked_add_days(chrono::Days::new(14))
            .unwrap_or(Local::now())
            .date_naive();

        Self {
            initial_date,
            last_date,
            initial_date_string: initial_date.to_string(),
            last_date_string: last_date.to_string(),
        }
    }
}

impl DateFilters {
    pub fn is_valid(&self) -> bool {
        if !check_date_format(&self.initial_date_string)
            || !check_date_format(&self.last_date_string)
        {
            return false;
        }

        if parse_date_to_naive_datetime(&self.initial_date_string).unwrap_or_default()
            > parse_date_to_naive_datetime(&self.last_date_string).unwrap_or_default()
        {
            return false;
        }

        true
    }
}

#[derive(Debug, Clone)]
pub enum ReservationsTextInputFields {
    InitialFilterDate,
    LastFilterDate,
}

#[derive(Debug, Clone)]
pub enum ReservationsListDirectionAction {
    Back,
    Forward,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks the parent to go back
    Back,
    /// Asks the parent to add a toast
    AddToast(Toast),
    /// Hotkey (Subscription) pressed
    Hotkey(Hotkey),

    /// Asks to update the current state of the list page
    LoadListPage,
    /// Callback after initial page loading, set's the  list state
    PageLoaded(Vec<Reservation>, Vec<Room>, DateFilters),

    /// Callback when using the text inputs of the reservations page
    TextInputUpdate(String, ReservationsTextInputFields),
    /// Callback after clicking one of the two arrows to go one day back/forward
    DirectionActionInput(ReservationsListDirectionAction),

    /// Add Reservation page messages
    AddReservation(add::Message),
    /// Edit Reservation page messages
    EditReservation(edit::Message),

    /// Opens the add reservation page for the selected date and room
    OpenAddReservation(NaiveDate, Room),
    /// Opens the edit reservation page for the reservation with the given id
    OpenEditReservation(i32),
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl Reservations {
    pub fn new(database: &Arc<Pool<Postgres>>) -> (Self, Task<Message>) {
        let dates = DateFilters::default();
        let database = database.clone();
        (
            Self {
                state: State::Loading,
            },
            Task::perform(
                async move {
                    let (reservations, rooms) = tokio::join!(
                        Reservation::get_all(database.clone(), dates.initial_date, dates.last_date),
                        Room::get_all(database.clone())
                    );
                    (reservations, rooms)
                },
                |(reservations, rooms)| match (reservations, rooms) {
                    (Ok(reservations), Ok(rooms)) => {
                        Message::PageLoaded(reservations, rooms, dates)
                    }
                    _ => Message::AddToast(Toast::error_toast(
                        "Error fetching reservations of rooms",
                    )),
                },
            ),
        )
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn update(
        &mut self,
        message: Message,
        database: &Arc<Pool<Postgres>>,
        now: Instant,
    ) -> Action {
        match message {
            Message::Back => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    match sub_screen {
                        SubScreen::List { .. } => return Action::Back,
                        SubScreen::Add(_) => {
                            return self.update(Message::LoadListPage, &database.clone(), now);
                        }
                        SubScreen::Edit(_) => {
                            return self.update(Message::LoadListPage, &database.clone(), now);
                        }
                    }
                }
                Action::None
            }
            Message::AddToast(toast) => Action::AddToast(toast),
            Message::Hotkey(hotkey) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List { .. } = sub_screen {
                        return match hotkey {
                            Hotkey::Tab(modifiers) => {
                                if modifiers.shift() {
                                    Action::Run(focus_previous())
                                } else {
                                    Action::Run(focus_next())
                                }
                            }
                        };
                    }
                }
                Action::None
            }
            Message::LoadListPage => {
                let date_filters = if let State::Ready {
                    sub_screen: SubScreen::List { date_filters, .. },
                    ..
                } = &mut self.state
                {
                    if date_filters.is_valid() {
                        date_filters.initial_date =
                            parse_date_to_naive_datetime(&date_filters.initial_date_string)
                                .unwrap_or_default()
                                .date();
                        date_filters.last_date =
                            parse_date_to_naive_datetime(&date_filters.last_date_string)
                                .unwrap_or_default()
                                .date();
                    }

                    date_filters
                } else {
                    &DateFilters::default()
                };

                let database = database.clone();
                let date_filters = date_filters.clone();

                Action::Run(Task::perform(
                    async move {
                        let (reservations, rooms) = tokio::join!(
                            Reservation::get_all(
                                database.clone(),
                                date_filters.initial_date,
                                date_filters.last_date
                            ),
                            Room::get_all(database.clone())
                        );
                        (reservations, rooms)
                    },
                    |(reservations, rooms)| match (reservations, rooms) {
                        (Ok(reservations), Ok(rooms)) => {
                            Message::PageLoaded(reservations, rooms, date_filters)
                        }
                        _ => Message::AddToast(Toast::error_toast(
                            "Error fetching reservations of rooms",
                        )),
                    },
                ))
            }
            Message::PageLoaded(reservations, rooms, date_filters) => {
                self.state = State::Ready {
                    sub_screen: SubScreen::List {
                        date_filters,
                        reservations,
                        rooms,
                    },
                };
                Action::None
            }
            Message::TextInputUpdate(new_value, field) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List { date_filters, .. } = sub_screen {
                        match field {
                            ReservationsTextInputFields::InitialFilterDate => {
                                date_filters.initial_date_string = new_value;
                            }
                            ReservationsTextInputFields::LastFilterDate => {
                                date_filters.last_date_string = new_value;
                            }
                        }
                    }
                }

                Action::None
            }
            Message::DirectionActionInput(action) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List { date_filters, .. } = sub_screen {
                        match action {
                            ReservationsListDirectionAction::Back => {
                                #[allow(clippy::collapsible_if)]
                                if let Some(new_initial_date) = date_filters
                                    .initial_date
                                    .checked_sub_days(chrono::Days::new(1))
                                {
                                    if let Some(new_last_date) = date_filters
                                        .last_date
                                        .checked_sub_days(chrono::Days::new(1))
                                    {
                                        date_filters.initial_date_string =
                                            new_initial_date.to_string();
                                        date_filters.last_date_string = new_last_date.to_string();
                                        date_filters.initial_date = parse_date_to_naive_datetime(
                                            &date_filters.initial_date_string,
                                        )
                                        .unwrap_or_default()
                                        .date();
                                        date_filters.last_date = parse_date_to_naive_datetime(
                                            &date_filters.last_date_string,
                                        )
                                        .unwrap_or_default()
                                        .date();

                                        return self.update(
                                            Message::LoadListPage,
                                            &database.clone(),
                                            now,
                                        );
                                    }
                                }
                            }
                            ReservationsListDirectionAction::Forward => {
                                #[allow(clippy::collapsible_if)]
                                if let Some(new_initial_date) = date_filters
                                    .initial_date
                                    .checked_add_days(chrono::Days::new(1))
                                {
                                    if let Some(new_last_date) = date_filters
                                        .last_date
                                        .checked_add_days(chrono::Days::new(1))
                                    {
                                        date_filters.initial_date_string =
                                            new_initial_date.to_string();
                                        date_filters.last_date_string = new_last_date.to_string();
                                        date_filters.initial_date = parse_date_to_naive_datetime(
                                            &date_filters.initial_date_string,
                                        )
                                        .unwrap_or_default()
                                        .date();
                                        date_filters.last_date = parse_date_to_naive_datetime(
                                            &date_filters.last_date_string,
                                        )
                                        .unwrap_or_default()
                                        .date();

                                        return self.update(
                                            Message::LoadListPage,
                                            &database.clone(),
                                            now,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            Message::AddReservation(message) => {
                let State::Ready { sub_screen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::Add(add) = sub_screen else {
                    return Action::None;
                };

                match add.update(message, database, now) {
                    add::Action::None => Action::None,
                    add::Action::Run(task) => Action::Run(task.map(Message::AddReservation)),
                    add::Action::Back => self.update(Message::LoadListPage, &database.clone(), now),
                    add::Action::AddToast(toast) => Action::AddToast(toast),
                }
            }
            Message::EditReservation(message) => {
                let State::Ready { sub_screen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::Edit(edit) = sub_screen else {
                    return Action::None;
                };

                match edit.update(message, database, now) {
                    edit::Action::None => Action::None,
                    edit::Action::Run(task) => Action::Run(task.map(Message::EditReservation)),
                    edit::Action::Back => {
                        self.update(Message::LoadListPage, &database.clone(), now)
                    }
                    edit::Action::AddToast(toast) => Action::AddToast(toast),
                }
            }
            Message::OpenAddReservation(initial_date, room) => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (add, task) = add::AddReservation::new(database, initial_date, room);
                *sub_screen = SubScreen::Add(add);
                Action::Run(task.map(Message::AddReservation))
            }
            Message::OpenEditReservation(reservation_id) => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (edit, task) = edit::EditReservation::new(database, reservation_id);
                *sub_screen = SubScreen::Edit(edit);
                Action::Run(task.map(Message::EditReservation))
            }
        }
    }

    pub fn view(&self, now: Instant) -> iced::Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading...")).center(Length::Fill).into(),
            State::Ready { sub_screen } => match sub_screen {
                SubScreen::List {
                    date_filters,
                    reservations,
                    rooms,
                } => list_screen(date_filters, reservations, rooms),
                SubScreen::Add(add) => add.view(now).map(Message::AddReservation),
                SubScreen::Edit(edit) => edit.view(now).map(Message::EditReservation),
            },
        }
    }

    pub fn subscription(&self, now: Instant) -> Subscription<Message> {
        let State::Ready { sub_screen, .. } = &self.state else {
            return Subscription::none();
        };

        match sub_screen {
            SubScreen::List { .. } => event::listen_with(handle_event),
            SubScreen::Add(add) => add.subscription(now).map(Message::AddReservation),
            SubScreen::Edit(edit) => edit.subscription(now).map(Message::EditReservation),
        }
    }
}

//
// SUBSCRIPTION HANDLING
//

#[derive(Debug, Clone)]
pub enum Hotkey {
    Tab(Modifiers),
}

fn handle_event(event: event::Event, _: event::Status, _: iced::window::Id) -> Option<Message> {
    match event {
        #[allow(clippy::collapsible_match)]
        event::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
            Key::Named(Named::Tab) => Some(Message::Hotkey(Hotkey::Tab(modifiers))),
            _ => None,
        },
        _ => None,
    }
}

//
// VIEW COMPOSING
//

// LIST SCREEN

fn list_screen<'a>(
    date_filters: &'a DateFilters,
    reservations: &'a [Reservation],
    rooms: &'a [Room],
) -> iced::Element<'a, Message> {
    let header = list_header(date_filters);
    let content = reservations_calendar(date_filters, reservations, rooms);

    column![header, content]
        .spacing(GLOBAL_SPACING)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn list_header<'a>(date_filters: &'a DateFilters) -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    // TODO: FIX DATE VALIDATION, NOW WE CAN ENTER WHATEVER...
    let initial_date_label =
        text(format!("{} (yyyy-mm-dd)", fl!("initial-date"))).width(Length::Fill);
    let initial_date_input = text_input(
        fl!("initial-date").as_str(),
        &date_filters.initial_date_string,
    )
    .on_input(|c| Message::TextInputUpdate(c, ReservationsTextInputFields::InitialFilterDate))
    .size(TEXT_SIZE)
    .width(Length::Fill);

    let last_date_label = text(format!("{} (yyyy-mm-dd)", fl!("last-date"))).width(Length::Fill);
    let last_date_input = text_input(fl!("last-date").as_str(), &date_filters.last_date_string)
        .on_input(|c| Message::TextInputUpdate(c, ReservationsTextInputFields::LastFilterDate))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    let submit_button = button(text(fl!("filter")).center().size(TEXT_SIZE))
        .on_press_maybe(date_filters.is_valid().then_some(Message::LoadListPage))
        .width(Length::Shrink)
        .height(GLOBAL_BUTTON_HEIGHT);

    let initial_date_input_column = column![initial_date_label, initial_date_input].spacing(1.);
    let last_date_input_column = column![last_date_label, last_date_input].spacing(1.);

    row![
        back_button,
        text(fl!("reservations")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        initial_date_input_column,
        last_date_input_column,
        submit_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}

fn reservations_calendar<'a>(
    date_filters: &'a DateFilters,
    reservations: &'a [Reservation],
    rooms: &'a [Room],
) -> iced::Element<'a, Message> {
    let cell_width = Length::Fill; // If I put a fixed width here I also have to put it on the Header Row or everything breaks
    let cell_height = Length::Fixed(GLOBAL_BUTTON_HEIGHT);

    // header row with days
    let mut header_row = Row::new();

    // top left action buttons
    header_row = header_row
        .push(
            Row::new()
                .push(
                    button(
                        text("<")
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                            .height(Length::Fill)
                            .width(Length::Fill),
                    )
                    .style(button::secondary)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .on_press(Message::DirectionActionInput(
                        ReservationsListDirectionAction::Back,
                    )),
                )
                .push(
                    button(
                        text(">")
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                            .height(Length::Fill)
                            .width(Length::Fill),
                    )
                    .style(button::secondary)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .on_press(Message::DirectionActionInput(
                        ReservationsListDirectionAction::Forward,
                    )),
                )
                .align_y(Alignment::Center)
                .spacing(GLOBAL_SPACING)
                .width(cell_width)
                .height(cell_height),
        )
        .width(Length::Fill)
        .spacing(GLOBAL_SPACING);

    // add each day of range as a header
    let mut current_date = date_filters.initial_date;
    while current_date <= date_filters.last_date {
        header_row = header_row.push(
            text(format!("{}/{}", current_date.day(), current_date.month()))
                .size(16)
                .center()
                .width(cell_width)
                .height(cell_height),
        );
        current_date += chrono::Duration::days(1);
    }

    // final calendar view
    let mut calendar_view = Column::new().push(header_row).spacing(GLOBAL_SPACING);

    for room in rooms {
        // each room is a row
        let mut row = Row::new().spacing(GLOBAL_SPACING);
        row = row.push(
            text(&room.name)
                .size(16)
                .center()
                .width(cell_width)
                .height(cell_height),
        );

        // loop through each day in the range and check for reservations
        let mut current_date = date_filters.initial_date;
        while current_date <= date_filters.last_date {
            let mut cell_content = container(
                button("")
                    .on_press(Message::OpenAddReservation(current_date, room.clone()))
                    .style(button::secondary)
                    .width(cell_width)
                    .height(cell_height),
            );

            for reservation in reservations {
                // check if the current room is part of the reservation and if the date falls within the reservation period
                if reservation.rooms.iter().any(|r| r.room_id == room.id)
                        && reservation.entry_date.unwrap_or_default().date() <= current_date
                        // departure date does not have an equal because we can book a room the day someone departs
                        && reservation.departure_date.unwrap_or_default().date() > current_date
                {
                    match reservation.occupied {
                        true => {
                            cell_content = container(Tooltip::new(
                                button("")
                                    .on_press(Message::OpenEditReservation(
                                        reservation.id.unwrap_or_default(),
                                    ))
                                    .style(button::success)
                                    .width(cell_width)
                                    .height(cell_height),
                                container(reservation.client_name.as_str())
                                    .style(container::rounded_box)
                                    .padding(3.),
                                tooltip::Position::FollowCursor,
                            ));
                        }
                        false => {
                            cell_content = container(Tooltip::new(
                                button("")
                                    .on_press(Message::OpenEditReservation(
                                        reservation.id.unwrap_or_default(),
                                    ))
                                    .style(button::danger)
                                    .width(cell_width)
                                    .height(cell_height),
                                container(reservation.client_name.as_str())
                                    .style(container::rounded_box)
                                    .padding(3.),
                                tooltip::Position::FollowCursor,
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

    container(scrollable(calendar_view)).into()
}
