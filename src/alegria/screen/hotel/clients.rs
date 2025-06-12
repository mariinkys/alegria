// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::time::Instant;
use iced::widget::{container, text};
use iced::{Length, Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::client::Client;
use crate::alegria::widgets::toast::Toast;

pub struct Clients {
    state: State,
}

enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

pub enum SubScreen {
    List,
    Upsert,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,

    SetClients(Vec<Client>),
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl Clients {
    pub fn new(_database: &Arc<Pool<Postgres>>) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
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
            Message::Back => todo!(),
            Message::SetClients(clients) => todo!(),
        }
        Action::None
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading...")).center(Length::Fill).into(),
            State::Ready { sub_screen } => match sub_screen {
                SubScreen::List => todo!(),
                SubScreen::Upsert => todo!(),
            },
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}
