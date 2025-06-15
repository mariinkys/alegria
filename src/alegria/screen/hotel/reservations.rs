// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use chrono::{Local, NaiveDate};
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::time::Instant;
use iced::widget::{Space, button, column, focus_next, focus_previous, row, text, text_input};
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
    Add,
    Edit,
}

#[derive(Debug, Clone)]
struct DateFilters {
    initial_date: String,
    last_date: String,
}

impl Default for DateFilters {
    fn default() -> Self {
        let initial_date = Local::now().date_naive().to_string();
        let last_date = Local::now()
            .checked_add_days(chrono::Days::new(14))
            .unwrap_or(Local::now())
            .date_naive()
            .to_string();

        Self {
            initial_date,
            last_date,
        }
    }
}

impl DateFilters {
    pub fn is_valid(&self) -> bool {
        if !check_date_format(&self.initial_date) || !check_date_format(&self.last_date) {
            return false;
        }

        true
    }

    pub fn get_initial_date(&self) -> NaiveDate {
        parse_date_to_naive_datetime(&self.initial_date)
            .unwrap_or_default()
            .date()
    }

    pub fn get_last_date(&self) -> NaiveDate {
        parse_date_to_naive_datetime(&self.last_date)
            .unwrap_or_default()
            .date()
    }
}

#[derive(Debug, Clone)]
pub enum ReservationsTextInputFields {
    InitialFilterDate,
    LastFilterDate,
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
    PageLoaded(Vec<Reservation>, Vec<Room>),

    /// Callback when using the text inputs of the reservations page
    TextInputUpdate(String, ReservationsTextInputFields),
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
        let initial_date = dates.get_initial_date();
        let last_date = dates.get_last_date();

        let database = database.clone();
        (
            Self {
                state: State::Loading,
            },
            Task::perform(
                async move {
                    let (reservations, rooms) = tokio::join!(
                        Reservation::get_all(database.clone(), initial_date, last_date),
                        Room::get_all(database.clone())
                    );
                    (reservations, rooms)
                },
                |(reservations, rooms)| match (reservations, rooms) {
                    (Ok(reservations), Ok(rooms)) => Message::PageLoaded(reservations, rooms),
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
                        SubScreen::Add => {
                            return self.update(Message::LoadListPage, &database.clone(), now);
                        }
                        SubScreen::Edit => {
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
                    if let SubScreen::Add | SubScreen::Edit = sub_screen {
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
                    date_filters
                } else {
                    &DateFilters::default()
                };

                let (initial_date, last_date) = (
                    date_filters.get_initial_date(),
                    date_filters.get_last_date(),
                );

                let database = database.clone();

                Action::Run(Task::perform(
                    async move {
                        let (reservations, rooms) = tokio::join!(
                            Reservation::get_all(database.clone(), initial_date, last_date),
                            Room::get_all(database.clone())
                        );
                        (reservations, rooms)
                    },
                    |(reservations, rooms)| match (reservations, rooms) {
                        (Ok(reservations), Ok(rooms)) => Message::PageLoaded(reservations, rooms),
                        _ => Message::AddToast(Toast::error_toast(
                            "Error fetching reservations of rooms",
                        )),
                    },
                ))
            }
            Message::PageLoaded(reservations, rooms) => {
                let date_filters = DateFilters::default();

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
                                date_filters.initial_date = new_value;
                            }
                            ReservationsTextInputFields::LastFilterDate => {
                                date_filters.last_date = new_value;
                            }
                        }
                    }
                }
                Action::None
            }
        }
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading...")).center(Length::Fill).into(),
            State::Ready { sub_screen } => match sub_screen {
                SubScreen::List {
                    date_filters,
                    reservations,
                    rooms,
                } => list_screen(date_filters, reservations, rooms),
                SubScreen::Add => todo!(),
                SubScreen::Edit => todo!(),
            },
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        event::listen_with(handle_event)
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
    let content = text("Content");

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

    let initial_date_label =
        text(format!("{} (yyyy-mm-dd)", fl!("initial-date"))).width(Length::Fill);
    let initial_date_input = text_input(fl!("initial-date").as_str(), &date_filters.initial_date)
        .on_input(|c| Message::TextInputUpdate(c, ReservationsTextInputFields::InitialFilterDate))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    let last_date_label = text(format!("{} (yyyy-mm-dd)", fl!("last-date"))).width(Length::Fill);
    let last_date_input = text_input(fl!("last-date").as_str(), &date_filters.last_date)
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
