// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::time::Instant;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Length, Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::widgets::toast::Toast;
use crate::fl;
use alegria_utils::styling::*;

mod clients;
mod reservations;
mod room_types;
mod rooms;

pub struct Hotel {
    state: State,
}

enum State {
    #[allow(dead_code)]
    Loading,
    Ready {
        sub_screen: SubScreen,
    },
}

pub enum SubScreen {
    Home,
    Clients(clients::Clients),
    RoomTypes(room_types::RoomTypes),
    Rooms(rooms::Rooms),
    Reservations(reservations::Reservations),
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,

    Clients(clients::Message),
    OpenClients,

    RoomTypes(room_types::Message),
    OpenRoomTypes,

    Rooms(rooms::Message),
    OpenRooms,

    Reservations(reservations::Message),
    OpenReservations,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl Hotel {
    pub fn new(_database: &Arc<Pool<Postgres>>) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Ready {
                    sub_screen: SubScreen::Home,
                },
            },
            Task::none(),
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

            Message::Clients(message) => {
                let State::Ready { sub_screen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::Clients(clients) = sub_screen else {
                    return Action::None;
                };

                match clients.update(message, database, now) {
                    clients::Action::None => Action::None,
                    clients::Action::Run(task) => Action::Run(task.map(Message::Clients)),
                    clients::Action::Back => {
                        *sub_screen = SubScreen::Home;
                        Action::None
                    }
                    clients::Action::AddToast(toast) => Action::AddToast(toast),
                    clients::Action::ClientSelected(_) => {
                        eprintln!("this should not happen, we can never reach this statement");
                        Action::None
                    }
                }
            }
            Message::OpenClients => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (clients, task) = clients::Clients::new(database, clients::PageMode::Normal);
                *sub_screen = SubScreen::Clients(clients);
                Action::Run(task.map(Message::Clients))
            }

            Message::RoomTypes(message) => {
                let State::Ready { sub_screen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::RoomTypes(room_types) = sub_screen else {
                    return Action::None;
                };

                match room_types.update(message, database, now) {
                    room_types::Action::None => Action::None,
                    room_types::Action::Run(task) => Action::Run(task.map(Message::RoomTypes)),
                    room_types::Action::Back => {
                        *sub_screen = SubScreen::Home;
                        Action::None
                    }
                    room_types::Action::AddToast(toast) => Action::AddToast(toast),
                }
            }
            Message::OpenRoomTypes => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (room_types, task) = room_types::RoomTypes::new(database);
                *sub_screen = SubScreen::RoomTypes(room_types);
                Action::Run(task.map(Message::RoomTypes))
            }

            Message::Rooms(message) => {
                let State::Ready { sub_screen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::Rooms(rooms) = sub_screen else {
                    return Action::None;
                };

                match rooms.update(message, database, now) {
                    rooms::Action::None => Action::None,
                    rooms::Action::Run(task) => Action::Run(task.map(Message::Rooms)),
                    rooms::Action::Back => {
                        *sub_screen = SubScreen::Home;
                        Action::None
                    }
                    rooms::Action::AddToast(toast) => Action::AddToast(toast),
                }
            }
            Message::OpenRooms => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (rooms, task) = rooms::Rooms::new(database);
                *sub_screen = SubScreen::Rooms(rooms);
                Action::Run(task.map(Message::Rooms))
            }

            Message::Reservations(message) => {
                let State::Ready { sub_screen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::Reservations(reservations) = sub_screen else {
                    return Action::None;
                };

                match reservations.update(message, database, now) {
                    reservations::Action::None => Action::None,
                    reservations::Action::Run(task) => Action::Run(task.map(Message::Reservations)),
                    reservations::Action::Back => {
                        *sub_screen = SubScreen::Home;
                        Action::None
                    }
                    reservations::Action::AddToast(toast) => Action::AddToast(toast),
                }
            }
            Message::OpenReservations => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (reservations, task) = reservations::Reservations::new(database);
                *sub_screen = SubScreen::Reservations(reservations);
                Action::Run(task.map(Message::Reservations))
            }
        }
    }

    pub fn view(&self, now: Instant) -> iced::Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading...")).center(Length::Fill).into(),
            State::Ready { sub_screen } => match sub_screen {
                SubScreen::Home => {
                    let header = header();
                    let home = home();

                    container(
                        column![header, home]
                            .spacing(GLOBAL_SPACING)
                            .height(Length::Fill)
                            .width(Length::Fill),
                    )
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .padding(3.)
                    .into()
                }
                SubScreen::Clients(clients) => clients.view(now).map(Message::Clients),
                SubScreen::RoomTypes(room_types) => room_types.view(now).map(Message::RoomTypes),
                SubScreen::Rooms(rooms) => rooms.view(now).map(Message::Rooms),
                SubScreen::Reservations(reservations) => {
                    reservations.view(now).map(Message::Reservations)
                }
            },
        }
    }

    pub fn subscription(&self, now: Instant) -> Subscription<Message> {
        let State::Ready { sub_screen, .. } = &self.state else {
            return Subscription::none();
        };

        match sub_screen {
            SubScreen::Home => Subscription::none(),
            SubScreen::Clients(clients) => clients.subscription(now).map(Message::Clients),
            SubScreen::RoomTypes(room_types) => {
                room_types.subscription(now).map(Message::RoomTypes)
            }
            SubScreen::Rooms(rooms) => rooms.subscription(now).map(Message::Rooms),
            SubScreen::Reservations(reservations) => {
                reservations.subscription(now).map(Message::Reservations)
            }
        }
    }
}

//
// VIEW COMPOSING
//

/// Returns the view of the header row of the hotel screen
fn header<'a>() -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("hotel"))
            .size(TITLE_TEXT_SIZE)
            .align_y(Alignment::Center)
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .into()
}

/// Returns the view of the homepage of the hotel screen
fn home<'a>() -> iced::Element<'a, Message> {
    let buttons_row = iced::widget::Row::new()
        .push(
            button(
                text(fl!("reservations"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::OpenReservations)
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .push(
            button(
                text(fl!("room-types"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::OpenRoomTypes)
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .push(
            button(
                text(fl!("rooms"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::OpenRooms)
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .push(
            button(
                text(fl!("clients"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::OpenClients)
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .spacing(5.);

    container(buttons_row).center(Length::Fill).into()
}
