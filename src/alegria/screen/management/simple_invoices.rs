// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::time::Instant;
use iced::widget::{
    Column, Row, Rule, Space, button, column, focus_next, focus_previous, row, scrollable, text,
};
use iced::{Alignment, Element, Length, Renderer, Subscription, Theme, event};
use iced::{Task, widget::container};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::simple_invoice::SimpleInvoice;
use crate::alegria::utils::styling::{
    GLOBAL_BUTTON_HEIGHT, GLOBAL_SPACING, TEXT_SIZE, TITLE_TEXT_SIZE,
};

use crate::{
    alegria::{
        utils::pagination::{PaginationAction, PaginationConfig},
        widgets::toast::Toast,
    },
    fl,
};

pub struct SimpleInvoices {
    state: State,
}

enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

pub enum SubScreen {
    List {
        pagination_state: PaginationConfig,
        simple_invoices: Vec<SimpleInvoice>,
    },
    Details {
        simple_invoice: Box<SimpleInvoice>,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks the parent to go back
    Back,
    /// Asks the parent to add a toast
    AddToast(Toast),
    /// Hotkey (Subscription) pressed
    Hotkey(Hotkey),

    /// Asks to update the current list of simple-invoices
    FetchSimpleInvoices,
    /// Callback after initial page loading, set's the simple-invoices list on the state
    PageLoaded(Vec<SimpleInvoice>),

    /// Try to go left or right a page
    PaginationAction(PaginationAction),

    /// Callback after asking to see the details of a simple invoice, searches the simple_invoice on the db
    AskDetailsSimpleInvoice(i32),
    /// Changes to the details screen with the given simple_invoice
    OpenDetailsScreen(Box<SimpleInvoice>),

    /// Tries to delete the current simple invoice
    DeleteCurrentSimpleInvoice,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl SimpleInvoices {
    pub fn new(database: &Arc<Pool<Postgres>>) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(SimpleInvoice::get_all(database.clone()), |res| match res {
                Ok(res) => Message::PageLoaded(res),
                Err(err) => {
                    eprintln!("{err}");
                    Message::AddToast(Toast::error_toast(err))
                }
            }),
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
                        SubScreen::Details { .. } => {
                            return self.update(
                                Message::FetchSimpleInvoices,
                                &database.clone(),
                                now,
                            );
                        }
                    }
                }
                Action::None
            }
            Message::AddToast(toast) => Action::AddToast(toast),
            Message::Hotkey(hotkey) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Details { .. } = sub_screen {
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
            Message::FetchSimpleInvoices => Action::Run(Task::perform(
                SimpleInvoice::get_all(database.clone()),
                |res| match res {
                    Ok(res) => Message::PageLoaded(res),
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err))
                    }
                },
            )),
            Message::PageLoaded(res) => {
                self.state = State::Ready {
                    sub_screen: SubScreen::List {
                        pagination_state: PaginationConfig::default(),
                        simple_invoices: res,
                    },
                };
                Action::None
            }
            Message::PaginationAction(pagination_action) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List {
                        simple_invoices,
                        pagination_state,
                        ..
                    } = sub_screen
                    {
                        match pagination_action {
                            PaginationAction::Up => {}
                            PaginationAction::Down => {}
                            PaginationAction::Back => {
                                if pagination_state.current_page > 0 {
                                    pagination_state.current_page -= 1;
                                }
                            }
                            PaginationAction::Forward => {
                                let next_page_start = (pagination_state.current_page + 1)
                                    * pagination_state.items_per_page;
                                if next_page_start
                                    < simple_invoices.len().try_into().unwrap_or_default()
                                {
                                    pagination_state.current_page += 1;
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            Message::AskDetailsSimpleInvoice(simple_invoice_id) => {
                let database = database.clone();
                Action::Run(Task::perform(
                    SimpleInvoice::get_single(database.clone(), simple_invoice_id),
                    |result| match result {
                        Ok(simple_invoice) => Message::OpenDetailsScreen(Box::from(simple_invoice)),
                        Err(err) => Message::AddToast(Toast::error_toast(err)),
                    },
                ))
            }
            Message::OpenDetailsScreen(simple_invoice) => {
                self.state = State::Ready {
                    sub_screen: SubScreen::Details { simple_invoice },
                };
                Action::None
            }
            Message::DeleteCurrentSimpleInvoice => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Details { simple_invoice, .. } = sub_screen {
                        return Action::Run(Task::perform(
                            SimpleInvoice::delete(
                                database.clone(),
                                simple_invoice.id.unwrap_or_default(),
                            ),
                            |res| match res {
                                Ok(_) => Message::FetchSimpleInvoices,
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

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading...")).center(Length::Fill).into(),
            State::Ready { sub_screen } => match sub_screen {
                SubScreen::List {
                    pagination_state,
                    simple_invoices,
                } => list_screen(pagination_state, simple_invoices),
                SubScreen::Details { simple_invoice } => details_screen(simple_invoice),
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
    pagination_state: &'a PaginationConfig,
    simple_invoices: &'a [SimpleInvoice],
) -> iced::Element<'a, Message> {
    let header = list_header();
    let grid: Element<'a, Message, Theme, Renderer> = if simple_invoices.is_empty() {
        container(text(fl!("no-simple-invoices")).size(TITLE_TEXT_SIZE))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding(50.)
            .into()
    } else {
        let title_row = Row::new()
            .push(
                text(fl!("id"))
                    .size(TITLE_TEXT_SIZE)
                    .width(100.)
                    .align_y(Alignment::Center),
            )
            .push(
                text(fl!("total-invoice"))
                    .size(TITLE_TEXT_SIZE)
                    .width(500.)
                    .align_x(Alignment::Start)
                    .align_y(Alignment::End),
            )
            .push(
                text(fl!("details"))
                    .size(TITLE_TEXT_SIZE)
                    .width(200.)
                    .align_y(Alignment::Center)
                    .align_x(Alignment::End),
            )
            .width(Length::Shrink)
            .align_y(Alignment::Center);

        // Calculate the indices for the current page
        let start_index: usize =
            pagination_state.current_page as usize * pagination_state.items_per_page as usize;
        let end_index = usize::min(
            start_index + pagination_state.items_per_page as usize,
            simple_invoices.len(),
        );

        let mut grid = Column::new()
            .push(title_row)
            .spacing(GLOBAL_SPACING)
            .width(Length::Shrink);

        for simple_invoice in &simple_invoices[start_index..end_index] {
            let row = Row::new()
                .push(
                    text(simple_invoice.id.unwrap_or_default())
                        .size(TEXT_SIZE)
                        .width(100.)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center),
                )
                .push(
                    text(format!("{:.2}€", simple_invoice.total_price()))
                        .size(TEXT_SIZE)
                        .width(500.)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center),
                )
                .push(
                    row![
                        Space::new(Length::Fill, Length::Shrink),
                        button(
                            text(fl!("details"))
                                .size(TEXT_SIZE)
                                .align_y(Alignment::Center)
                        )
                        .on_press(Message::AskDetailsSimpleInvoice(simple_invoice.id.unwrap()))
                        .width(Length::Shrink)
                    ]
                    .width(200.),
                )
                .align_y(Alignment::Center);

            // Limit Rule size to sum of all column widths
            grid = grid.push(row![Rule::horizontal(1.)].width(800.));
            grid = grid.push(row);
        }

        scrollable(grid).spacing(GLOBAL_SPACING).into()
    };

    let page_controls = Column::new()
        .push(row![Rule::horizontal(1.)].width(800.))
        .push(
            text(format!(
                "{} {}",
                fl!("page").as_str(),
                &pagination_state.current_page + 1
            ))
            .align_x(Alignment::Center),
        )
        .push(
            Row::new()
                .width(800.)
                .push(
                    button(
                        text(fl!("back"))
                            .center()
                            .width(Length::Fill)
                            .height(GLOBAL_BUTTON_HEIGHT),
                    )
                    .on_press(Message::PaginationAction(PaginationAction::Back)),
                )
                .push(
                    button(
                        text(fl!("next"))
                            .center()
                            .width(Length::Fill)
                            .height(GLOBAL_BUTTON_HEIGHT),
                    )
                    .on_press(Message::PaginationAction(PaginationAction::Forward)),
                )
                .align_y(Alignment::Center)
                .spacing(GLOBAL_SPACING),
        )
        .spacing(GLOBAL_SPACING)
        .align_x(Alignment::Center);

    let content = container(
        column![grid, page_controls]
            .spacing(GLOBAL_SPACING)
            .width(800.),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center)
    .padding(50.);

    column![header, content]
        .spacing(GLOBAL_SPACING)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn list_header<'a>() -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("simple-invoices")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink)
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}

// DETAILS SCREEN

fn details_screen<'a>(simple_invoice: &'a SimpleInvoice) -> iced::Element<'a, Message> {
    let header = details_header(simple_invoice);

    let content = text("Content");

    column![
        header,
        container(content)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .padding(50.)
    ]
    .into()
}

fn details_header<'a>(simple_invoice: &'a SimpleInvoice) -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    let delete_button = button(text(fl!("delete")).center())
        .style(button::danger)
        .on_press_maybe(
            simple_invoice
                .id
                .map(|_| Message::DeleteCurrentSimpleInvoice),
        )
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("simple-invoices")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        delete_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}
