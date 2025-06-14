// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::advanced::graphics::core::Element;
use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::time::Instant;
use iced::widget::{
    Column, Row, Rule, Space, button, column, container, focus_next, focus_previous, pick_list,
    row, scrollable, text, text_input,
};
use iced::{Alignment, Length, Renderer, Subscription, Task, Theme, event};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::client::Client;
use crate::alegria::utils::date::parse_date_to_naive_datetime;
use crate::alegria::utils::entities::gender::Gender;
use crate::alegria::utils::entities::identity_document_type::IdentityDocumentType;
use crate::alegria::utils::pagination::{PaginationAction, PaginationConfig};
use crate::alegria::utils::styling::*;
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
        client: Box<Client>,
    },
}

#[derive(Debug, Clone)]
pub enum ClientTextInputFields {
    IdentityDocument,
    Name,
    FirstSurname,
    SecondSurname,
    Address,
    PostalCode,
    City,
    Province,
    Country,
    Nationality,
    PhoneNumber,
    MobilePhone,
    IdentityDocumentExpeditionDate,
    IdentityDocumentExpirationDate,
    Birthdate,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks the parent to go back
    Back,
    /// Asks the parent to add a toast
    AddToast(Toast),
    /// Hotkey (Subscription) pressed
    Hotkey(Hotkey),

    /// Asks to update the current list of clients
    FetchClients,
    /// Callback after initial page loading, set's the client list on the state
    PageLoaded(Vec<Client>),

    /// Try to go left or right a page on the ClientsList
    ClientsPaginationAction(PaginationAction),
    /// Callback after writing on the search box
    SearchUpdate(String),
    /// Callback after pressing enter on the search bar              
    SubmitSearch,
    /// Callback after clicking on the clear search button                         
    ClearSearch,

    /// Callback after asking to edit a client, searches the client on the db
    AskEditClient(i32),
    /// Changes the upsert screen with the given client
    OpenUpsertScreen(Box<Client>),

    /// Callback when using the text inputs to add or edit a client
    TextInputUpdate(String, ClientTextInputFields),
    /// Callback after selecting a new DocumentType for the current client
    UpdatedSelectedDocumentType(IdentityDocumentType),
    /// Callback after selecting a new Gender for the current client
    UpdatedSelectedGender(Gender),

    /// Tries to Add or Edit the current client to the database
    UpsertCurrentClient,
    /// Callback after upserting the client on the database
    UpsertedCurrentClient,
    /// Tries to delete the current Client
    DeleteCurrentClient,
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
            Message::Back => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    match sub_screen {
                        SubScreen::List { .. } => return Action::Back,
                        SubScreen::Upsert { .. } => {
                            return self.update(Message::FetchClients, &database.clone(), now);
                        }
                    }
                }
                Action::None
            }
            Message::AddToast(toast) => Action::AddToast(toast),
            Message::Hotkey(hotkey) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { .. } = sub_screen {
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
            Message::SearchUpdate(value) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List { current_search, .. } = sub_screen {
                        *current_search = value;
                    }
                }

                Action::None
            }
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
            Message::ClearSearch => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List {
                        current_search,
                        pagination_state,
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
            Message::AskEditClient(client_id) => Action::Run(Task::perform(
                Client::get_single(database.clone(), client_id),
                |res| match res {
                    Ok(res) => Message::OpenUpsertScreen(Box::from(res)),
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err))
                    }
                },
            )),
            Message::OpenUpsertScreen(client) => {
                self.state = State::Ready {
                    sub_screen: SubScreen::Upsert { client },
                };
                Action::None
            }
            Message::TextInputUpdate(new_value, field) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { client, .. } = sub_screen {
                        match field {
                            ClientTextInputFields::IdentityDocument => {
                                client.identity_document = new_value
                            }
                            ClientTextInputFields::Name => client.name = new_value,
                            ClientTextInputFields::FirstSurname => client.first_surname = new_value,
                            ClientTextInputFields::SecondSurname => {
                                client.second_surname = new_value
                            }
                            ClientTextInputFields::Address => client.address = new_value,
                            ClientTextInputFields::PostalCode => client.postal_code = new_value,
                            ClientTextInputFields::City => client.city = new_value,
                            ClientTextInputFields::Province => client.province = new_value,
                            ClientTextInputFields::Country => client.country = new_value,
                            ClientTextInputFields::Nationality => client.nationality = new_value,
                            ClientTextInputFields::PhoneNumber => client.phone_number = new_value,
                            ClientTextInputFields::MobilePhone => client.mobile_phone = new_value,
                            ClientTextInputFields::IdentityDocumentExpeditionDate => {
                                client.identity_document_expedition_date_string = new_value
                            }
                            ClientTextInputFields::IdentityDocumentExpirationDate => {
                                client.identity_document_expiration_date_string = new_value
                            }
                            ClientTextInputFields::Birthdate => client.birthdate_string = new_value,
                        }
                    }
                }
                Action::None
            }
            Message::UpdatedSelectedDocumentType(new_doc_type) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { client, .. } = sub_screen {
                        client.identity_document_type = Some(new_doc_type);
                    }
                }
                Action::None
            }
            Message::UpdatedSelectedGender(new_gender) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { client, .. } = sub_screen {
                        client.gender = Some(new_gender);
                    }
                }
                Action::None
            }
            Message::UpsertCurrentClient => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { client, .. } = sub_screen {
                        #[allow(clippy::collapsible_if)]
                        if client.is_valid() {
                            client.birthdate =
                                parse_date_to_naive_datetime(&client.birthdate_string);
                            client.identity_document_expedition_date = parse_date_to_naive_datetime(
                                &client.identity_document_expedition_date_string,
                            );
                            client.identity_document_expiration_date = parse_date_to_naive_datetime(
                                &client.identity_document_expiration_date_string,
                            );

                            return match client.id {
                                Some(_id) => Action::Run(Task::perform(
                                    Client::edit(database.clone(), *client.clone()),
                                    |res| match res {
                                        Ok(_) => Message::UpsertedCurrentClient,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::AddToast(Toast::error_toast(err))
                                        }
                                    },
                                )),
                                None => Action::Run(Task::perform(
                                    Client::add(database.clone(), *client.clone()),
                                    |res| match res {
                                        Ok(_) => Message::UpsertedCurrentClient,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::AddToast(Toast::error_toast(err))
                                        }
                                    },
                                )),
                            };
                        }
                    }
                }
                Action::None
            }
            Message::UpsertedCurrentClient => {
                self.update(Message::FetchClients, &database.clone(), now)
            }
            Message::DeleteCurrentClient => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { client, .. } = sub_screen {
                        return Action::Run(Task::perform(
                            Client::delete(database.clone(), client.id.unwrap_or_default()),
                            |res| match res {
                                Ok(_) => Message::FetchClients,
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
                    current_search,
                    pagination_state,
                    clients,
                } => list_screen(current_search, pagination_state, clients),
                SubScreen::Upsert { client } => upsert_screen(client),
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
    current_search: &'a str,
    pagination_state: &'a PaginationConfig,
    clients: &'a [Client],
) -> iced::Element<'a, Message> {
    let header = list_header();
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
                            .on_press(Message::AskEditClient(client.id.unwrap()))
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

        scrollable(grid).spacing(GLOBAL_SPACING).into()
    };

    let page_controls = Column::new()
        .push(row![Rule::horizontal(1.)].width(850.))
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
                .align_y(Alignment::Center)
                .spacing(GLOBAL_SPACING),
        )
        .spacing(GLOBAL_SPACING)
        .align_x(Alignment::Center);

    let content = container(
        column![search_bar, grid, page_controls]
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
        .on_press(Message::OpenUpsertScreen(Box::from(Client::default())))
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

// UPSERT SCREEN

fn upsert_screen<'a>(client: &'a Client) -> iced::Element<'a, Message> {
    let header = upsert_header(client);

    // First Name
    let name_label = text(format!("{}*", fl!("name"))).width(Length::Fill);
    let name_input = text_input(fl!("name").as_str(), &client.name)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Name))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // First Surname
    let first_surname_label = text(fl!("first-surname")).width(Length::Fill);
    let first_surname_input = text_input(fl!("first-surname").as_str(), &client.first_surname)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::FirstSurname))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Second Surname
    let second_surname_label = text(fl!("second-surname")).width(Length::Fill);
    let second_surname_input = text_input(fl!("second-surname").as_str(), &client.second_surname)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::SecondSurname))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Gender
    let gender_label = text(fl!("gender")).width(Length::Fill);
    let selected_gender = Gender::ALL
        .iter()
        .find(|&g| *g == client.gender.unwrap_or_default());
    let gender_selector =
        pick_list(Gender::ALL, selected_gender, Message::UpdatedSelectedGender).width(Length::Fill);

    //  BirthDate TODO: DatePicker
    let birthdate_date_label =
        text(format!("{} (yyyy-mm-dd)", fl!("birthdate"))).width(Length::Fill);
    let birthdate_input = text_input(fl!("birthdate").as_str(), &client.birthdate_string)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Birthdate))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Identity Document
    let identity_document_label =
        text(format!("{}*", fl!("identity-document"))).width(Length::Fill);
    let identity_document_input =
        text_input(fl!("identity-document").as_str(), &client.identity_document)
            .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::IdentityDocument))
            .size(TEXT_SIZE)
            .width(Length::Fill);

    // Identity Document Type
    let identity_document_type_label = text(fl!("identity-document-type")).width(Length::Fill);
    let selected_identity_document_type = IdentityDocumentType::ALL
        .iter()
        .find(|&idt| *idt == client.identity_document_type.unwrap_or_default());
    let identity_document_type_selector = pick_list(
        IdentityDocumentType::ALL,
        selected_identity_document_type,
        Message::UpdatedSelectedDocumentType,
    )
    .width(Length::Fill);

    // Identity Document Expedition Date TODO: Date Picker
    let identity_document_expedition_date_label = text(format!(
        "{} (yyyy-mm-dd)",
        fl!("identity-document-expedition-date")
    ))
    .width(Length::Fill);
    let identity_document_expedition_date_input = text_input(
        fl!("identity-document-expedition-date").as_str(),
        &client.identity_document_expedition_date_string,
    )
    .on_input(|c| {
        Message::TextInputUpdate(c, ClientTextInputFields::IdentityDocumentExpeditionDate)
    })
    .size(TEXT_SIZE)
    .width(Length::Fill);

    // Identity Document Expiration Date TODO: Date Picker
    let identity_document_expiration_date_label = text(format!(
        "{} (yyyy-mm-dd)",
        fl!("identity-document-expiration-date")
    ))
    .width(Length::Fill);
    let identity_document_expiration_date_input = text_input(
        fl!("identity-document-expiration-date").as_str(),
        &client.identity_document_expiration_date_string,
    )
    .on_input(|c| {
        Message::TextInputUpdate(c, ClientTextInputFields::IdentityDocumentExpirationDate)
    })
    .size(TEXT_SIZE)
    .width(Length::Fill);

    // Address
    let address_label = text(fl!("address")).width(Length::Fill);
    let address_input = text_input(fl!("address").as_str(), &client.address)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Address))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Postal Code
    let postal_code_label = text(fl!("postal-code")).width(Length::Fill);
    let postal_code_input = text_input(fl!("postal-code").as_str(), &client.postal_code)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::PostalCode))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // City
    let city_label = text(fl!("city")).width(Length::Fill);
    let city_input = text_input(fl!("city").as_str(), &client.city)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::City))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Province
    let province_label = text(fl!("province")).width(Length::Fill);
    let province_input = text_input(fl!("province").as_str(), &client.province)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Province))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Country
    let country_label = text(format!("{}*", fl!("country"))).width(Length::Fill);
    let country_input = text_input(fl!("country").as_str(), &client.country)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Country))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Nationality
    let nationality_label = text(fl!("nationality")).width(Length::Fill);
    let nationality_input = text_input(fl!("nationality").as_str(), &client.nationality)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Nationality))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Phone Number
    let phone_number_label = text(fl!("phone-number")).width(Length::Fill);
    let phone_number_input = text_input(fl!("phone-number").as_str(), &client.phone_number)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::PhoneNumber))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Mobile Phone
    let mobile_phone_label = text(fl!("mobile-phone")).width(Length::Fill);
    let mobile_phone_input = text_input(fl!("mobile-phone").as_str(), &client.mobile_phone)
        .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::MobilePhone))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Submit
    let submit_button_text = if client.id.is_some() {
        text(fl!("edit"))
    } else {
        text(fl!("add"))
    };
    let submit_button = button(submit_button_text.center().size(TEXT_SIZE))
        .on_press_maybe(client.is_valid().then_some(Message::UpsertCurrentClient))
        .width(Length::Fill);

    // Input Columns
    let name_input_column = Column::new()
        .push(name_label)
        .push(name_input)
        .width(Length::Fill)
        .spacing(1.);

    let first_surname_input_column = Column::new()
        .push(first_surname_label)
        .push(first_surname_input)
        .width(Length::Fill)
        .spacing(1.);

    let second_surname_input_column = Column::new()
        .push(second_surname_label)
        .push(second_surname_input)
        .width(Length::Fill)
        .spacing(1.);

    let gender_input_column = Column::new()
        .push(gender_label)
        .push(gender_selector)
        .width(Length::Fill)
        .spacing(1.);

    let birthdate_input_column = Column::new()
        .push(birthdate_date_label)
        .push(birthdate_input)
        .width(Length::Fill)
        .spacing(1.);

    let identity_document_column = Column::new()
        .push(identity_document_label)
        .push(identity_document_input)
        .width(Length::Fill)
        .spacing(1.);

    let identity_document_type_input_column = Column::new()
        .push(identity_document_type_label)
        .push(identity_document_type_selector)
        .width(Length::Fill)
        .spacing(1.);

    let identity_document_expedition_date_column = Column::new()
        .push(identity_document_expedition_date_label)
        .push(identity_document_expedition_date_input)
        .width(Length::Fill)
        .spacing(1.);

    let identity_document_expiration_date_column = Column::new()
        .push(identity_document_expiration_date_label)
        .push(identity_document_expiration_date_input)
        .width(Length::Fill)
        .spacing(1.);

    let address_input_column = Column::new()
        .push(address_label)
        .push(address_input)
        .width(Length::Fill)
        .spacing(1.);

    let postal_code_input_column = Column::new()
        .push(postal_code_label)
        .push(postal_code_input)
        .width(Length::Fill)
        .spacing(1.);

    let city_input_column = Column::new()
        .push(city_label)
        .push(city_input)
        .width(Length::Fill)
        .spacing(1.);

    let province_input_column = Column::new()
        .push(province_label)
        .push(province_input)
        .width(Length::Fill)
        .spacing(1.);

    let country_input_column = Column::new()
        .push(country_label)
        .push(country_input)
        .width(Length::Fill)
        .spacing(1.);

    let nationality_input_column = Column::new()
        .push(nationality_label)
        .push(nationality_input)
        .width(Length::Fill)
        .spacing(1.);

    let phone_number_input_column = Column::new()
        .push(phone_number_label)
        .push(phone_number_input)
        .width(Length::Fill)
        .spacing(1.);

    let mobile_phone_input_column = Column::new()
        .push(mobile_phone_label)
        .push(mobile_phone_input)
        .width(Length::Fill)
        .spacing(1.);

    let form_column = Column::new()
        .push(name_input_column)
        .push(
            Row::new()
                .push(first_surname_input_column)
                .push(second_surname_input_column)
                .spacing(GLOBAL_SPACING)
                .width(850.),
        )
        .push(
            Row::new()
                .push(gender_input_column)
                .push(birthdate_input_column)
                .spacing(GLOBAL_SPACING)
                .width(850.),
        )
        .push(
            Row::new()
                .push(identity_document_column)
                .push(identity_document_type_input_column)
                .spacing(GLOBAL_SPACING)
                .width(850.),
        )
        .push(
            Row::new()
                .push(identity_document_expedition_date_column)
                .push(identity_document_expiration_date_column)
                .spacing(GLOBAL_SPACING)
                .width(850.),
        )
        .push(address_input_column)
        .push(
            Row::new()
                .push(postal_code_input_column)
                .push(city_input_column)
                .push(province_input_column)
                .spacing(GLOBAL_SPACING)
                .width(850.),
        )
        .push(
            Row::new()
                .push(country_input_column)
                .push(nationality_input_column)
                .spacing(GLOBAL_SPACING)
                .width(850.),
        )
        .push(
            Row::new()
                .push(phone_number_input_column)
                .push(mobile_phone_input_column)
                .spacing(GLOBAL_SPACING)
                .width(850.),
        )
        .push(submit_button)
        .width(850.)
        .spacing(GLOBAL_SPACING);

    column![
        header,
        container(form_column)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .padding(50.)
    ]
    .into()
}

fn upsert_header<'a>(client: &'a Client) -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    let delete_button = button(text(fl!("delete")).center())
        .style(button::danger)
        .on_press_maybe(client.id.map(|_| Message::DeleteCurrentClient))
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("client")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        delete_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}
