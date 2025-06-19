// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::Task;
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::time::Instant;
use iced::widget::{
    Column, Row, button, checkbox, column, container, focus_next, focus_previous, pick_list, row,
    text, text_input,
};
use iced::{Alignment, Length, Subscription, event};
use sqlx::{Pool, Postgres};

use crate::alegria::screen::hotel::clients::{self, Clients};
use alegria_core::models::reservation::Reservation;
use alegria_core::models::room::Room;
use alegria_core::models::sold_room::SoldRoom;
use alegria_utils::date::parse_date_to_naive_datetime;
use alegria_utils::styling::{GLOBAL_BUTTON_HEIGHT, GLOBAL_SPACING, TEXT_SIZE, TITLE_TEXT_SIZE};

use crate::{alegria::widgets::toast::Toast, fl};

pub struct AddReservation {
    state: State,
}

enum State {
    Loading,
    // We need to preserve the state of the add screen when we open the client selection
    Ready {
        sub_screen: SubScreen,
        reservation: Box<Reservation>,
        rooms: Arc<Vec<Room>>,
        reservations: Vec<Reservation>,
    },
}

pub enum SubScreen {
    None,
    ClientsSelection(Clients),
}

#[derive(Debug, Clone)]
pub enum InputFields {
    EntryDate,
    DepartureDate,
    Occupied,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks the parent to go back
    Back,
    /// Asks the parent to add a toast
    AddToast(Toast),
    /// Hotkey (Subscription) pressed
    Hotkey(Hotkey),

    /// Callback after initial page load
    PageLoaded(Box<Reservation>, Arc<Vec<Room>>, Vec<Reservation>),

    /// Callback when using the form inputs
    FormInputUpdate(String, InputFields),
    /// Asks to add a room to the vec of booked rooms of the current add reservation
    AddReservationRoom(i32, Option<f32>),
    /// Asks to remove a room to the vec of booked rooms of the current add reservation
    RemoveReservationRoom(i32),
    /// Asks to open the client selector page/component
    OpenClientSelector,
    /// Messages of the clients (selector) page
    Clients(clients::Message),
    /// Tries to add the current reservation to the database
    AddReservation,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl AddReservation {
    pub fn new(
        database: &Arc<Pool<Postgres>>,
        rooms: Arc<Vec<Room>>,
        reservation: Reservation,
    ) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(
                Reservation::get_all(
                    database.clone(),
                    reservation.entry_date.unwrap().date(),
                    reservation
                        .entry_date
                        .unwrap()
                        .date()
                        .checked_add_days(chrono::Days::new(120)) // is this a sensible number of dates to check?
                        .unwrap_or_default(),
                ),
                |res| match res {
                    Ok(reservations) => {
                        Message::PageLoaded(Box::from(reservation), rooms, reservations)
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err))
                    }
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
            Message::Back => Action::Back,
            Message::AddToast(toast) => Action::AddToast(toast),
            Message::Hotkey(hotkey) => {
                if let State::Ready { .. } = &mut self.state {
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
                Action::None
            }
            Message::PageLoaded(reservation, rooms, reservations) => {
                self.state = State::Ready {
                    sub_screen: SubScreen::None,
                    reservation,
                    rooms,
                    reservations,
                };
                Action::None
            }
            Message::FormInputUpdate(new_value, field) => {
                if let State::Ready { reservation, .. } = &mut self.state {
                    match field {
                        InputFields::EntryDate => {
                            reservation.entry_date_string = new_value;
                        }
                        InputFields::DepartureDate => {
                            reservation.departure_date_string = new_value;
                        }
                        InputFields::Occupied => {
                            reservation.occupied = !reservation.occupied;
                        }
                    }
                }
                Action::None
            }
            Message::AddReservationRoom(room_id, room_price) => {
                if let State::Ready { reservation, .. } = &mut self.state {
                    let room_already_exists = reservation
                        .rooms
                        .iter()
                        .any(|sold_room| sold_room.room_id == Some(room_id));

                    if !room_already_exists {
                        reservation.rooms.push(SoldRoom {
                            id: None,
                            room_id: Some(room_id),
                            guests: Vec::new(),
                            price: room_price,
                            invoices: Vec::new(),
                            room_name: Box::from(""),
                        });
                    }
                }
                Action::None
            }
            Message::RemoveReservationRoom(room_id) => {
                if let State::Ready { reservation, .. } = &mut self.state {
                    reservation
                        .rooms
                        .retain(|room| room.room_id != Some(room_id) || !room.invoices.is_empty());
                }
                Action::None
            }
            Message::Clients(message) => {
                let State::Ready {
                    sub_screen,
                    reservation,
                    ..
                } = &mut self.state
                else {
                    return Action::None;
                };

                let SubScreen::ClientsSelection(clients_selector_page) = sub_screen else {
                    return Action::None;
                };

                match clients_selector_page.update(message, &database.clone(), now) {
                    clients::Action::None => Action::None,
                    clients::Action::Back => {
                        if let State::Ready { sub_screen, .. } = &mut self.state {
                            *sub_screen = SubScreen::None;
                        }
                        Action::None
                    }
                    clients::Action::Run(task) => Action::Run(task.map(Message::Clients)),
                    clients::Action::AddToast(toast) => Action::AddToast(toast),
                    clients::Action::ClientSelected(client) => {
                        reservation.client_id = client.id;
                        reservation.client_name = format!(
                            "{} {} {} | {}",
                            client.name,
                            client.first_surname,
                            client.second_surname,
                            client.country
                        );
                        if let State::Ready { sub_screen, .. } = &mut self.state {
                            *sub_screen = SubScreen::None;
                        }
                        Action::None
                    }
                }
            }
            Message::OpenClientSelector => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (clients, task) = clients::Clients::new(database, clients::PageMode::Select);
                *sub_screen = SubScreen::ClientsSelection(clients);
                Action::Run(task.map(Message::Clients))
            }
            Message::AddReservation => {
                if let State::Ready { reservation, .. } = &mut self.state {
                    #[allow(clippy::collapsible_if)]
                    if reservation.is_valid() {
                        // since we validated we can unwrap the date, we know it's valid
                        reservation.entry_date = Some(
                            parse_date_to_naive_datetime(&reservation.entry_date_string)
                                .unwrap()
                                .date()
                                .and_hms_opt(0, 0, 0)
                                .unwrap(),
                        );
                        reservation.departure_date = Some(
                            parse_date_to_naive_datetime(&reservation.departure_date_string)
                                .unwrap()
                                .date()
                                .and_hms_opt(0, 0, 0)
                                .unwrap(),
                        );

                        if reservation.entry_date.unwrap().date()
                            >= reservation.departure_date.unwrap().date()
                        {
                            return Action::AddToast(Toast::error_toast(
                                "Entry date must not be greater than departure date",
                            ));
                        }

                        return Action::Run(Task::perform(
                            Reservation::add(database.clone(), *reservation.clone()),
                            |res| match res {
                                Ok(_) => Message::Back,
                                Err(err) => {
                                    eprintln!("{err}");
                                    Message::AddToast(Toast::error_toast(err))
                                }
                            },
                        ));
                    }
                }

                Action::None
            }
        }
    }

    pub fn view(&self, now: Instant) -> iced::Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading...")).center(Length::Fill).into(),
            State::Ready {
                sub_screen,
                reservation,
                rooms,
                reservations,
            } => match sub_screen {
                SubScreen::None => add_form(reservation, rooms, reservations),
                SubScreen::ClientsSelection(clients) => clients.view(now).map(Message::Clients),
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

fn add_form<'a>(
    reservation: &'a Reservation,
    rooms: &'a [Room],
    reservations: &'a [Reservation],
) -> iced::Element<'a, Message> {
    let header = header();
    let content = form_content(reservation, rooms, reservations);

    column![
        header,
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
    ]
    .spacing(GLOBAL_SPACING)
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

fn header<'a>() -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    row![back_button, text(fl!("reservations")).size(TITLE_TEXT_SIZE),]
        .align_y(Alignment::Center)
        .spacing(GLOBAL_SPACING)
        .padding(3.)
        .into()
}

fn form_content<'a>(
    new_reservation: &'a Reservation,
    rooms: &'a [Room],
    reservations: &'a [Reservation],
) -> iced::Element<'a, Message> {
    let entry_date_label = text(format!("{} (yyyy-mm-dd)", fl!("entry-date"))).width(Length::Fill);
    let entry_date_input = text_input(
        fl!("entry-date").as_str(),
        &new_reservation.entry_date_string,
    )
    .on_input(|c| Message::FormInputUpdate(c, InputFields::EntryDate))
    .size(TEXT_SIZE)
    .width(Length::Fill);

    let departure_date_label =
        text(format!("{} (yyyy-mm-dd)", fl!("departure-date"))).width(Length::Fill);
    let departure_date_input = text_input(
        fl!("departure-date").as_str(),
        &new_reservation.departure_date_string,
    )
    .on_input(|c| Message::FormInputUpdate(c, InputFields::DepartureDate))
    .size(TEXT_SIZE)
    .width(Length::Fill);

    let occupied = checkbox(fl!("occupied"), new_reservation.occupied)
        .text_size(TEXT_SIZE)
        .on_toggle(|_| Message::FormInputUpdate(String::new(), InputFields::Occupied));

    // Rooms Selector
    let available_rooms = rooms
        .iter()
        .filter(|room| {
            !reservations.iter().any(|reservation| {
                reservation.rooms.iter().any(|r| r.room_id == room.id)
                    && reservation.entry_date.unwrap() < new_reservation.departure_date.unwrap()
                    && reservation.departure_date.unwrap() > new_reservation.entry_date.unwrap()
            })
        })
        .cloned()
        .collect::<Vec<Room>>();
    let rooms_label = text(fl!("rooms")).width(Length::Fill);
    let selected_room = available_rooms.first().cloned();
    let rooms_selector = pick_list(available_rooms, selected_room, |r| {
        Message::AddReservationRoom(r.id.unwrap(), r.default_room_price)
    })
    .width(Length::Fill);

    // Already Selected Rooms
    let mut reservation_rooms_column = Column::new()
        .push(text(fl!("rooms")))
        .width(Length::Fill)
        .spacing(GLOBAL_SPACING);
    for sold_room in &new_reservation.rooms {
        let room = rooms.iter().find(|r| r.id == sold_room.room_id);
        if let Some(room) = room {
            reservation_rooms_column = reservation_rooms_column.push(
                Row::new()
                    .push(text(&room.name).width(Length::Fill))
                    .push(
                        button("X")
                            .on_press(Message::RemoveReservationRoom(room.id.unwrap_or_default())),
                    )
                    .align_y(Alignment::Center)
                    .width(Length::Fill),
            )
        }
    }

    // Client Selection
    let client_text = if new_reservation.client_name.is_empty() {
        fl!("no-client-selected")
    } else {
        new_reservation.client_name.clone()
    };
    let client_row = row![
        text(client_text).width(Length::Fill),
        button(text(fl!("select")).center()).on_press(Message::OpenClientSelector)
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING);

    // Submit
    let submit_button = button(text(fl!("add")).width(Length::Fill).center())
        .on_press_maybe(
            new_reservation
                .is_valid()
                .then_some(Message::AddReservation),
        )
        .width(Length::Fill);

    let entry_date_input_column = column![entry_date_label, entry_date_input]
        .width(850.)
        .spacing(1.);
    let departure_date_input_column = column![departure_date_label, departure_date_input]
        .width(850.)
        .spacing(1.);
    let rooms_input_column = row![
        column![rooms_label, rooms_selector].width(425.).spacing(1.),
        reservation_rooms_column.width(425.)
    ]
    .width(850.)
    .spacing(GLOBAL_SPACING);
    let client_selection_column = column![text(fl!("main-client")).width(Length::Fill), client_row]
        .width(850.)
        .spacing(1.);

    Column::new()
        .push(client_selection_column)
        .push(entry_date_input_column)
        .push(departure_date_input_column)
        .push(occupied)
        .push(rooms_input_column)
        .push(submit_button)
        .width(850.)
        .spacing(GLOBAL_SPACING)
        .into()
}
