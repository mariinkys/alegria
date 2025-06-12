// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::advanced::graphics::core::Element;
use iced::time::Instant;
use iced::widget::{Column, Row, Rule, Space, button, column, container, row, text, text_input};
use iced::{Alignment, Length, Renderer, Subscription, Task, Theme};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::client::Client;
use crate::alegria::utils::pagination::{PaginationAction, PaginationConfig};
use crate::alegria::widgets::toast::Toast;
use crate::fl;

pub struct Clients {
    state: State,
}

enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

pub enum SubScreen {
    List {
        current_search: String,
        pagination_state: PaginationConfig,
        clients: Vec<Client>,
    },
    Upsert {
        client: Client,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    AddToast(Toast),

    FetchClients,
    PageLoaded(Vec<Client>),

    ClientsPaginationAction(PaginationAction), // Try to go left or right a page on the ClientsList
    SearchUpdate(String),                      // Callback after writing on the search box
    SubmitSearch,                              // Callback after pressing enter on the search bar
    ClearSearch,                               // Callback after clicking on the clear search button
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl Clients {
    pub fn new(database: &Arc<Pool<Postgres>>) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(Client::get_all(database.clone()), |res| match res {
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
            Message::Back => Action::Back,
            Message::AddToast(toast) => Action::AddToast(toast),

            Message::FetchClients => Action::Run(Task::perform(
                Client::get_all(database.clone()),
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
                        clients: res,
                        current_search: String::new(),
                    },
                };
                Action::None
            }

            // Try to go left or right a page on the ClientsList
            Message::ClientsPaginationAction(pagination_action) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List {
                        clients,
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
                                if next_page_start < clients.len().try_into().unwrap_or_default() {
                                    pagination_state.current_page += 1;
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            // Callback after writing on the search box
            Message::SearchUpdate(value) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List { current_search, .. } = sub_screen {
                        *current_search = value;
                    }
                }

                Action::None
            }
            // Callback after pressing enter on the search bar
            Message::SubmitSearch => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List {
                        current_search,
                        pagination_state,
                        clients,
                        ..
                    } = sub_screen
                    {
                        if current_search.is_empty() {
                            return self.update(Message::FetchClients, &database.clone(), now);
                        } else if !clients.is_empty() {
                            *clients = clients
                                .iter()
                                .filter(|client| {
                                    client
                                        .search_field
                                        .to_lowercase()
                                        .contains(&current_search.to_lowercase())
                                })
                                .cloned()
                                .collect();
                        }

                        *pagination_state = Default::default();
                    }
                }

                Action::None
            }
            // Callback after clicking on the clear search button
            Message::ClearSearch => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List {
                        current_search,
                        pagination_state,
                        clients,
                        ..
                    } = sub_screen
                    {
                        *current_search = String::new();
                        *pagination_state = Default::default();
                        return self.update(Message::FetchClients, &database.clone(), now);
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
                    current_search,
                    pagination_state,
                    clients,
                } => list_screen(current_search, pagination_state, clients),
                SubScreen::Upsert { client } => todo!(),
            },
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}

//
// VIEW COMPOSING
//

const TITLE_TEXT_SIZE: f32 = 25.0;
const GLOBAL_SPACING: f32 = 6.;
const GLOBAL_BUTTON_HEIGHT: f32 = 60.;
const TEXT_SIZE: f32 = 18.0;

fn list_screen<'a>(
    current_search: &'a String,
    pagination_state: &'a PaginationConfig,
    clients: &'a [Client],
) -> iced::Element<'a, Message> {
    let header = list_header();
    let grid: Element<'a, Message, Theme, Renderer> = if clients.is_empty() {
        container(text(fl!("no-clients")).size(TITLE_TEXT_SIZE))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding(50.)
            .into()
    } else {
        let title_row = Row::new()
            .push(
                text(fl!("name"))
                    .size(TITLE_TEXT_SIZE)
                    .width(250.)
                    .align_y(Alignment::Center),
            )
            .push(
                text("TD")
                    .size(TITLE_TEXT_SIZE)
                    .width(100.)
                    .align_y(Alignment::Center),
            )
            .push(
                text(fl!("identity-document"))
                    .size(TITLE_TEXT_SIZE)
                    .width(200.)
                    .align_y(Alignment::Center),
            )
            .push(
                text(fl!("country"))
                    .size(TITLE_TEXT_SIZE)
                    .width(150.)
                    .align_y(Alignment::Center),
            )
            .push(
                text(fl!("edit"))
                    .size(TITLE_TEXT_SIZE)
                    .width(150.)
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
            clients.len(),
        );

        let mut grid = Column::new()
            .push(title_row)
            .align_x(Alignment::Center)
            .spacing(GLOBAL_SPACING)
            .width(Length::Shrink);

        for client in &clients[start_index..end_index] {
            let row = Row::new()
                .width(Length::Shrink)
                .push(
                    text(format!(
                        "{} {} {}",
                        &client.name, &client.first_surname, &client.second_surname
                    ))
                    .size(TEXT_SIZE)
                    .width(250.)
                    .align_y(Alignment::Center),
                )
                .push(
                    text(client.identity_document_type.unwrap_or_default())
                        .size(TEXT_SIZE)
                        .width(100.)
                        .align_y(Alignment::Center),
                )
                .push(
                    text(&client.identity_document)
                        .size(TEXT_SIZE)
                        .width(200.)
                        .align_y(Alignment::Center),
                )
                .push(
                    text(&client.country)
                        .size(TEXT_SIZE)
                        .width(150.)
                        .align_y(Alignment::Center),
                )
                .push(
                    container(
                        button(text(fl!("edit")).size(TEXT_SIZE).align_y(Alignment::Center))
                            //.on_press(Message::AskEditClient(client.id.unwrap()))
                            .width(Length::Shrink),
                    )
                    .width(150.)
                    .align_x(Alignment::End)
                    .align_y(Alignment::Center),
                )
                .align_y(Alignment::Center);

            // Limit Rule size to sum of all column widths
            grid = grid.push(row![Rule::horizontal(1.)].width(850.));
            grid = grid.push(row);
        }

        grid = grid.push(row![Rule::horizontal(1.)].width(850.));
        grid = grid.push(text(format!(
            "{} {}",
            fl!("page").as_str(),
            &pagination_state.current_page + 1
        )));
        grid = grid.push(Space::with_height(Length::Fill));
        grid = grid.push(
            Row::new()
                .width(850.)
                .push(
                    button(
                        text(fl!("back"))
                            .center()
                            .width(Length::Fill)
                            .height(GLOBAL_BUTTON_HEIGHT),
                    )
                    .on_press(Message::ClientsPaginationAction(PaginationAction::Back)),
                )
                .push(
                    button(
                        text(fl!("next"))
                            .center()
                            .width(Length::Fill)
                            .height(GLOBAL_BUTTON_HEIGHT),
                    )
                    .on_press(Message::ClientsPaginationAction(PaginationAction::Forward)),
                )
                .spacing(GLOBAL_SPACING),
        );

        grid.into()
    };
    let search_bar = Row::new()
        .push(
            text_input(fl!("search").as_str(), current_search)
                .on_input(Message::SearchUpdate)
                .on_submit(Message::SubmitSearch)
                .size(TEXT_SIZE)
                .width(Length::Fill),
        )
        .push(
            button(
                text(fl!("clear"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .size(TEXT_SIZE),
            )
            .on_press(Message::ClearSearch)
            .width(Length::Shrink),
        )
        .spacing(GLOBAL_SPACING)
        .width(850.);

    let content = container(
        column![search_bar, grid]
            .spacing(GLOBAL_SPACING)
            .width(850.),
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

    let add_button = button(text(fl!("add")).center())
        //.on_press(Message::OpenUpsertScreen)
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("clients")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        add_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}
