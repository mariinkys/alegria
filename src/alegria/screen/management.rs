// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::time::Instant;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Length, Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::utils::styling::*;
use crate::alegria::widgets::toast::Toast;
use crate::fl;

mod product_categories;
mod products;
mod simple_invoices;

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
    Products(products::Products),
    ProductCategories(product_categories::ProductCategories),
    SimpleInvoices(simple_invoices::SimpleInvoices),
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    Products(products::Message),
    OpenProducts,
    ProductCategories(product_categories::Message),
    OpenProductCategories,
    SimpleInvoices(simple_invoices::Message),
    OpenSimpleInvoices,
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
            Message::Products(message) => {
                let State::Ready { sub_screen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::Products(products) = sub_screen else {
                    return Action::None;
                };

                match products.update(message, database, now) {
                    products::Action::None => Action::None,
                    products::Action::Run(task) => Action::Run(task.map(Message::Products)),
                    products::Action::Back => {
                        *sub_screen = SubScreen::Home;
                        Action::None
                    }
                    products::Action::AddToast(toast) => Action::AddToast(toast),
                }
            }
            Message::OpenProducts => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (products, task) = products::Products::new(database);
                *sub_screen = SubScreen::Products(products);
                Action::Run(task.map(Message::Products))
            }
            Message::ProductCategories(message) => {
                let State::Ready { sub_screen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::ProductCategories(product_categories) = sub_screen else {
                    return Action::None;
                };

                match product_categories.update(message, database, now) {
                    product_categories::Action::None => Action::None,
                    product_categories::Action::Run(task) => {
                        Action::Run(task.map(Message::ProductCategories))
                    }
                    product_categories::Action::Back => {
                        *sub_screen = SubScreen::Home;
                        Action::None
                    }
                    product_categories::Action::AddToast(toast) => Action::AddToast(toast),
                }
            }
            Message::OpenProductCategories => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (product_categories, task) =
                    product_categories::ProductCategories::new(database);
                *sub_screen = SubScreen::ProductCategories(product_categories);
                Action::Run(task.map(Message::ProductCategories))
            }
            Message::SimpleInvoices(message) => {
                let State::Ready { sub_screen } = &mut self.state else {
                    return Action::None;
                };

                let SubScreen::SimpleInvoices(simple_invoices) = sub_screen else {
                    return Action::None;
                };

                match simple_invoices.update(message, database, now) {
                    simple_invoices::Action::None => Action::None,
                    simple_invoices::Action::Run(task) => {
                        Action::Run(task.map(Message::SimpleInvoices))
                    }
                    simple_invoices::Action::Back => {
                        *sub_screen = SubScreen::Home;
                        Action::None
                    }
                    simple_invoices::Action::AddToast(toast) => Action::AddToast(toast),
                }
            }
            Message::OpenSimpleInvoices => {
                let State::Ready { sub_screen, .. } = &mut self.state else {
                    return Action::None;
                };

                let (simple_invoices, task) = simple_invoices::SimpleInvoices::new(database);
                *sub_screen = SubScreen::SimpleInvoices(simple_invoices);
                Action::Run(task.map(Message::SimpleInvoices))
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
                SubScreen::Products(products) => products.view(now).map(Message::Products),
                SubScreen::ProductCategories(product_categories) => {
                    product_categories.view(now).map(Message::ProductCategories)
                }
                SubScreen::SimpleInvoices(simple_invoices) => {
                    simple_invoices.view(now).map(Message::SimpleInvoices)
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
            SubScreen::Products(products) => products.subscription(now).map(Message::Products),
            SubScreen::ProductCategories(product_categories) => product_categories
                .subscription(now)
                .map(Message::ProductCategories),
            SubScreen::SimpleInvoices(simple_invoices) => simple_invoices
                .subscription(now)
                .map(Message::SimpleInvoices),
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
            .on_press(Message::OpenProducts)
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .push(
            button(
                text(fl!("product-categories"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::OpenProductCategories)
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .push(
            button(
                text(fl!("simple-invoices"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::OpenSimpleInvoices)
            .width(SQUAREBUTTONXY)
            .height(SQUAREBUTTONXY),
        )
        .spacing(5.);

    container(buttons_row).center(Length::Fill).into()
}
