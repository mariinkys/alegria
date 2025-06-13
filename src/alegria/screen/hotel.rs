// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::time::Instant;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Length, Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::widgets::toast::Toast;
use crate::fl;

mod clients;

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
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,

    Clients(clients::Message),
    OpenClients,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

const TITLE_TEXT_SIZE: f32 = 25.0;
const GLOBAL_SPACING: f32 = 6.;
const GLOBAL_BUTTON_HEIGHT: f32 = 60.;
const SQUAREBUTTONXY: f32 = 120.;

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
                }
            }
            Message::OpenClients => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (clients, task) = clients::Clients::new(database);
                *sub_screen = SubScreen::Clients(clients);
                Action::Run(task.map(Message::Clients))
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
            //.on_press(Message::OpenReservations)
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .push(
            button(
                text(fl!("room-types"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .push(
            button(
                text(fl!("rooms"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
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
