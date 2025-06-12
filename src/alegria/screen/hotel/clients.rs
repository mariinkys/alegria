// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::time::Instant;
use iced::widget::{container, text};
use iced::{Length, Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::client::Client;
use crate::alegria::utils::pagination::{PaginationAction, PaginationConfig};
use crate::alegria::widgets::toast::Toast;

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
    Upsert,
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
                                // This aberration happens since adding the printpdf crate which added the deranged crate that causes this
                                if next_page_start
                                    < <usize as std::convert::TryInto<i32>>::try_into(clients.len())
                                        .unwrap_or_default()
                                {
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
                } => todo!(),
                SubScreen::Upsert => todo!(),
            },
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}
