// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::time::Instant;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Length, Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::utils::styling::*;
use crate::alegria::widgets::toast::Toast;
use crate::fl;

pub struct Management {
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
    //Products(clients::Clients),
    //ProductTypes(room_types::RoomTypes),
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    //Products(clients::Message),
    //OpenProducts,

    //ProductTypes(room_types::Message),
    //OpenProductTypes,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl Management {
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
                } //SubScreen::Products(products) => products.view(now).map(Message::Products),
                  //SubScreen::ProductTypes(product_types) => product_types.view(now).map(Message::ProductTypes),
            },
        }
    }

    pub fn subscription(&self, now: Instant) -> Subscription<Message> {
        let State::Ready { sub_screen, .. } = &self.state else {
            return Subscription::none();
        };

        match sub_screen {
            SubScreen::Home => Subscription::none(),
            // SubScreen::Products(products) => products.subscription(now).map(Message::Products),
            // SubScreen::ProductTypes(prduct_types) => {
            //     product_types.subscription(now).map(Message::ProductTypes)
            // }
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
        text(fl!("management"))
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
                text(fl!("products"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            //.on_press(Message::OpenProducts)
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .push(
            button(
                text(fl!("product-types"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            //.on_press(Message::OpenProductTypes)
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .spacing(5.);

    container(buttons_row).center(Length::Fill).into()
}
