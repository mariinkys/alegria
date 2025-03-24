// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use iced::{
    Alignment, Element, Length, Pixels, Task,
    widget::{
        Column, Row, Rule, Space, button, column, container, pick_list, row, text, text_input,
    },
};
use iced_aw::{
    DatePicker,
    date_picker::{self, Date},
};
use sqlx::PgPool;

use crate::{
    alegria::{
        action::AlegriaAction,
        core::models::{
            client::Client, gender::Gender, identity_document_type::IdentityDocumentType,
        },
        utils::{check_date_format, parse_date_to_naive_datetime},
    },
    fl,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ClientsPageMode {
    Normal,
    Selection,
}

#[derive(Debug, Clone, PartialEq)]
enum ClientsScreen {
    List,
    AddEdit,
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
pub enum ClientDateInputFields {
    IdentityDocumentExpeditionDate,
    IdentityDocumentExpirationDate,
    Birthdate,
}

/// Holds the pagination state (generic, for various entities)
#[derive(Debug, Clone)]
pub struct PaginationConfig {
    items_per_page: i32,
    current_page: i32,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        PaginationConfig {
            items_per_page: 10,
            current_page: 0,
        }
    }
}

/// Identifies a pagination action
#[derive(Debug, Clone)]
pub enum PaginationAction {
    Back,
    Forward,
}

pub struct Clients {
    /// Database of the application
    pub database: Option<Arc<PgPool>>,
    /// Determines which is the current mode of the subscreen
    pub page_mode: ClientsPageMode,
    /// Determines which is the current view of the subscreen
    current_screen: ClientsScreen,
    /// Holds the state of all the clients
    clients: Vec<Client>,
    /// Holds the state of all the identity document types, needed for the document type selector
    identity_document_types: Vec<IdentityDocumentType>,
    /// Holds the state of all the genders, needed for the gender selector
    genders: Vec<Gender>,
    /// Holds the state of the current editing/adding Client
    add_edit_client: Option<Client>,
    /// Controls wether or not the expedition date picker should be shown
    show_expedition_date_picker: bool,
    /// Controls wether or not the expiration date picker should be shown
    show_expiration_date_picker: bool,
    /// Controls wether or not the birthdate date picker should be shown
    show_birthdate_date_picker: bool,
    /// Holds the pagination state and config for the clients list
    clients_pagination_state: PaginationConfig,
    /// Search Bar value
    current_search: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,                        // Asks the parent (app.rs) to go back
    ClientSelected(Box<Client>), // Tells the parent a client has been selected

    FetchClients,            // Fetches all the current clients
    SetClients(Vec<Client>), // Sets the clients on the app state

    InitPage, // Intended to be called from Hotel when first opening the page, asks for the necessary data and executes the appropiate callbacks
    SetIdentityDocumentTypes(Vec<IdentityDocumentType>), // Sets the identity document types on the app state
    SetGenders(Vec<Gender>), // Sets the identity document types on the app state

    SetAddEditClient(Box<Client>), // changes the screen and the add_edit_client state (after asking for an edit Client)
    AskEditClient(i32), // Callback after asking to edit a client, searches the client on the db
    AskSelectClient(i32), // Callback after asking to select a client, searches the client on the db and emmits the parent callback
    AskAddClient, // Callback after asking to add a room, changes the screen and the add_edit_client state
    CancelClientOperation, // Callback after asking to cancel an add or an edit

    TextInputUpdate(String, ClientTextInputFields), // Callback when using the text inputs to add or edit a client
    UpdatedSelectedDocumentTypeId(i32), // Callback after selecting a new DocumentTypeId for the current client
    UpdatedSelectedGenderId(i32), // Callback after selecting a new GenderId for the current client
    ShowDatePicker(ClientDateInputFields), // Toggles the variable to show the specified date picker
    CancelDateOperation,          // Cancels the datepicker changes
    UpdateDateField(date_picker::Date, ClientDateInputFields), // Callback after submiting a new date via datepicker

    AddCurrentClient,      // Tries to Add the current client to the database
    EditCurrentClient,     // Tries to Edit the current client on the database
    DeleteCurrentClient,   // Tries to delete the current Client
    ModifiedCurrentClient, // Callback after delete/update/add of a current Client

    ClientsPaginationAction(PaginationAction), // Try to go left or right a page on the ClientsList
    SearchUpdate(String),                      // Callback after writing on the search box
    SubmitSearch,                              // Callback after pressing enter on the search bar
    ClearSearch,                               // Callback after clicking on the clear search button
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum ClientsInstruction {
    Back,                        // Asks the parent (app.rs) to go back
    ClientSelected(Box<Client>), // Tells the parent a client has been selected
}

impl Default for Clients {
    fn default() -> Self {
        Self {
            database: None,
            page_mode: ClientsPageMode::Normal,
            current_screen: ClientsScreen::List,
            clients: Vec::new(),
            identity_document_types: Vec::new(),
            genders: Vec::new(),
            add_edit_client: None,
            show_expedition_date_picker: false,
            show_expiration_date_picker: false,
            show_birthdate_date_picker: false,
            clients_pagination_state: PaginationConfig::default(),
            current_search: String::new(),
        }
    }
}

impl Clients {
    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<ClientsInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            Message::Back => {
                action.add_instruction(ClientsInstruction::Back);
            }
            // Tells the parent a client has been selected
            Message::ClientSelected(client) => {
                if self.page_mode == ClientsPageMode::Selection {
                    action.add_instruction(ClientsInstruction::ClientSelected(client));
                }
            }

            // Fetches all the current clients
            Message::FetchClients => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        Client::get_all(pool.clone()),
                        |res| match res {
                            Ok(res) => Message::SetClients(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetClients(Vec::new())
                            }
                        },
                    ));
                }
            }
            // Sets the clients on the app state
            Message::SetClients(res) => {
                self.clients = res;
            }

            Message::InitPage => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        Client::get_all(pool.clone()),
                        |res| match res {
                            Ok(res) => Message::SetClients(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetClients(Vec::new())
                            }
                        },
                    ));

                    action.add_task(Task::perform(
                        IdentityDocumentType::get_all(pool.clone()),
                        |res| match res {
                            Ok(res) => Message::SetIdentityDocumentTypes(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetIdentityDocumentTypes(Vec::new())
                            }
                        },
                    ));

                    action.add_task(Task::perform(
                        Gender::get_all(pool.clone()),
                        |res| match res {
                            Ok(res) => Message::SetGenders(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetGenders(Vec::new())
                            }
                        },
                    ));
                }
            }
            // Sets the identity document types on the app state
            Message::SetIdentityDocumentTypes(res) => {
                self.identity_document_types = res;
            }
            // Sets the genders on the app state
            Message::SetGenders(res) => {
                self.genders = res;
            }

            // changes the screen and the add_edit_client state (after asking for an edit Client)
            Message::SetAddEditClient(client) => {
                self.add_edit_client = Some(*client);
                self.current_screen = ClientsScreen::AddEdit;
            }
            // Callback after asking to edit a client, searches the client on the db
            Message::AskEditClient(client_id) => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        Client::get_single(pool.clone(), client_id),
                        |res| match res {
                            Ok(res) => Message::SetAddEditClient(Box::new(res)),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetAddEditClient(Box::default())
                            }
                        },
                    ));
                }
            }
            // Callback after asking to select a client, searches the client on the db and emmits the parent callback
            Message::AskSelectClient(client_id) => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        Client::get_single(pool.clone(), client_id),
                        |res| match res {
                            Ok(res) => Message::ClientSelected(Box::new(res)),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::ClientSelected(Box::default())
                            }
                        },
                    ));
                }
            }
            // Callback after asking to edit a client, changes the screen and the add_edit_client state
            Message::AskAddClient => {
                self.add_edit_client = Some(Client::default());
                self.current_screen = ClientsScreen::AddEdit;
            }
            // Callback after asking to cancel an add or an edit
            Message::CancelClientOperation => {
                self.add_edit_client = None;
                self.current_screen = ClientsScreen::List;
                return self.update(Message::FetchClients);
            }

            // Callback when using the text inputs to add or edit a client
            Message::TextInputUpdate(new_value, field) => {
                if let Some(client) = self.add_edit_client.as_mut() {
                    match field {
                        ClientTextInputFields::IdentityDocument => {
                            client.identity_document = new_value
                        }
                        ClientTextInputFields::Name => client.name = new_value,
                        ClientTextInputFields::FirstSurname => client.first_surname = new_value,
                        ClientTextInputFields::SecondSurname => client.second_surname = new_value,
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
            // Callback after selecting a new DocumentTypeId for the current client
            Message::UpdatedSelectedDocumentTypeId(new_id) => {
                if let Some(client) = &mut self.add_edit_client {
                    client.identity_document_type_id = Some(new_id);
                }
            }
            // Callback after selecting a new GenderId for the current client
            Message::UpdatedSelectedGenderId(new_id) => {
                if let Some(client) = &mut self.add_edit_client {
                    client.gender_id = Some(new_id);
                }
            }
            // Toggles the variable to show the specified date picker
            Message::ShowDatePicker(date_field) => match date_field {
                ClientDateInputFields::IdentityDocumentExpeditionDate => {
                    self.show_expedition_date_picker = true;
                }
                ClientDateInputFields::IdentityDocumentExpirationDate => {
                    self.show_expiration_date_picker = true;
                }
                ClientDateInputFields::Birthdate => {
                    self.show_birthdate_date_picker = true;
                }
            },
            // Cancels the datepicker changes
            Message::CancelDateOperation => {
                self.show_expedition_date_picker = false;
                self.show_expiration_date_picker = false;
                self.show_birthdate_date_picker = false;
            }
            // Callback after submiting a new date via datepicker
            Message::UpdateDateField(iced_aw_date, field) => {
                if let Some(client) = &mut self.add_edit_client {
                    let new_date = NaiveDate::from_ymd_opt(
                        iced_aw_date.year,
                        iced_aw_date.month,
                        iced_aw_date.day,
                    );

                    match new_date {
                        Some(date) => {
                            match field {
                                ClientDateInputFields::IdentityDocumentExpeditionDate => {
                                    client.identity_document_expedition_date =
                                        Some(NaiveDateTime::new(date, NaiveTime::MIN));
                                    client.identity_document_expedition_date_string =
                                        format!("{}-{}-{}", date.year(), date.month(), date.day())
                                }
                                ClientDateInputFields::IdentityDocumentExpirationDate => {
                                    client.identity_document_expiration_date =
                                        Some(NaiveDateTime::new(date, NaiveTime::MIN));
                                    client.identity_document_expiration_date_string =
                                        format!("{}-{}-{}", date.year(), date.month(), date.day())
                                }
                                ClientDateInputFields::Birthdate => {
                                    client.birthdate =
                                        Some(NaiveDateTime::new(date, NaiveTime::MIN));
                                    client.birthdate_string =
                                        format!("{}-{}-{}", date.year(), date.month(), date.day())
                                }
                            }
                            self.show_expedition_date_picker = false;
                            self.show_expiration_date_picker = false;
                            self.show_birthdate_date_picker = false;
                        }
                        None => {
                            eprintln!("Could not parse new date");
                            match field {
                                ClientDateInputFields::IdentityDocumentExpeditionDate => {
                                    client.identity_document_expedition_date = None
                                }
                                ClientDateInputFields::IdentityDocumentExpirationDate => {
                                    client.identity_document_expiration_date = None
                                }
                                ClientDateInputFields::Birthdate => client.birthdate = None,
                            }
                        }
                    }
                }
            }

            // Tries to Add the current client to the database
            Message::AddCurrentClient => {
                if let Some(client) = &mut self.add_edit_client {
                    if is_client_valid(client) && client.id.is_none() {
                        client.birthdate = parse_date_to_naive_datetime(&client.birthdate_string);
                        client.identity_document_expedition_date = parse_date_to_naive_datetime(
                            &client.identity_document_expedition_date_string,
                        );
                        client.identity_document_expiration_date = parse_date_to_naive_datetime(
                            &client.identity_document_expiration_date_string,
                        );

                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                Client::add(pool.clone(), client.clone()),
                                |res| match res {
                                    Ok(_) => Message::ModifiedCurrentClient,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::CancelClientOperation
                                    }
                                },
                            ));
                        }
                    }
                }
            }
            // Tries to Edit the current client on the database
            Message::EditCurrentClient => {
                if let Some(client) = &mut self.add_edit_client {
                    if is_client_valid(client) && client.id.is_some() {
                        client.birthdate = parse_date_to_naive_datetime(&client.birthdate_string);
                        client.identity_document_expedition_date = parse_date_to_naive_datetime(
                            &client.identity_document_expedition_date_string,
                        );
                        client.identity_document_expiration_date = parse_date_to_naive_datetime(
                            &client.identity_document_expiration_date_string,
                        );

                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                Client::edit(pool.clone(), client.clone()),
                                |res| match res {
                                    Ok(_) => Message::ModifiedCurrentClient,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::CancelClientOperation
                                    }
                                },
                            ));
                        }
                    }
                }
            }
            // Tries to delete the current Client
            Message::DeleteCurrentClient => {
                if let Some(client) = &self.add_edit_client {
                    if client.id.is_some() {
                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                Client::delete(pool.clone(), client.id.unwrap_or_default()),
                                |res| match res {
                                    Ok(_) => Message::ModifiedCurrentClient,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::CancelClientOperation
                                    }
                                },
                            ));
                        }
                    }
                }
            }
            // Callback after add/update/delete of the current Client
            Message::ModifiedCurrentClient => {
                self.add_edit_client = None;
                self.current_screen = ClientsScreen::List;
                return self.update(Message::FetchClients);
            }

            // Try to go left or right a page on the ClientsList
            Message::ClientsPaginationAction(action) => match action {
                PaginationAction::Back => {
                    if self.clients_pagination_state.current_page > 0 {
                        self.clients_pagination_state.current_page -= 1;
                    }
                }
                PaginationAction::Forward => {
                    let next_page_start = (self.clients_pagination_state.current_page + 1)
                        * self.clients_pagination_state.items_per_page;
                    // This aberration happens since adding the printpdf crate which added the deranged crate that causes this
                    if next_page_start
                        < <usize as std::convert::TryInto<i32>>::try_into(self.clients.len())
                            .unwrap_or_default()
                    {
                        self.clients_pagination_state.current_page += 1;
                    }
                }
            },
            // Callback after writing on the search box
            Message::SearchUpdate(value) => {
                self.current_search = value;
            }
            // Callback after pressing enter on the search bar
            Message::SubmitSearch => {
                if self.current_search.is_empty() {
                    return self.update(Message::FetchClients);
                } else if !self.clients.is_empty() {
                    self.clients = self
                        .clients
                        .iter()
                        .filter(|client| {
                            client
                                .search_field
                                .to_lowercase()
                                .contains(&self.current_search.to_lowercase())
                        })
                        .cloned()
                        .collect();
                }

                self.clients_pagination_state = Default::default();
            }
            // Callback after clicking on the clear search button
            Message::ClearSearch => {
                self.current_search = String::new();
                self.clients_pagination_state = Default::default();
                return self.update(Message::FetchClients);
            }
        };

        action
    }

    const GLOBAL_SPACING: f32 = 6.;
    const GLOBAL_BUTTON_HEIGHT: f32 = 60.;

    /// Returns the view of the subscreen
    pub fn view(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        // HEADER
        let header_row = self.view_header_row();

        // ROOM TYPES CONTENT
        let content = match &self.current_screen {
            ClientsScreen::List => {
                let grid = self.view_clients_grid();

                let search_bar = Row::new()
                    .push(
                        text_input(fl!("search").as_str(), &self.current_search)
                            .on_input(Message::SearchUpdate)
                            .on_submit(Message::SubmitSearch)
                            .size(Self::TEXT_SIZE)
                            .width(Length::Fill),
                    )
                    .push(
                        button(
                            text(fl!("clear"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center)
                                .size(Self::TEXT_SIZE),
                        )
                        .on_press(Message::ClearSearch)
                        .width(Length::Shrink),
                    )
                    .spacing(spacing)
                    .width(850.);

                let content = column![search_bar, grid].spacing(spacing).width(850.);

                container(content)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(50.)
                    .into()
            }
            ClientsScreen::AddEdit => self.view_add_edit_screen(),
        };

        column![header_row, content]
            .spacing(spacing)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    //
    //  VIEW COMPOSING
    //

    const TITLE_TEXT_SIZE: f32 = 25.0;
    const TEXT_SIZE: f32 = 18.0;

    /// Returns the view of the header row of the subscreen
    fn view_header_row(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        let back_button = button(
            text(fl!("back"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .on_press(Message::Back)
        .height(button_height);

        let add_cancel_button = match &self.current_screen {
            ClientsScreen::List => button(
                text(fl!("add"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::AskAddClient)
            .height(button_height),
            ClientsScreen::AddEdit => button(
                text(fl!("cancel"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .style(button::danger)
            .on_press(Message::CancelClientOperation)
            .height(button_height),
        };

        let delete_button = button(
            text(fl!("delete"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .style(button::secondary)
        .on_press(Message::DeleteCurrentClient)
        .height(button_height);

        let mut result_row = Row::new();
        if self.current_screen == ClientsScreen::List {
            result_row = result_row.push(back_button);
        }

        result_row = result_row
            .push(
                text(fl!("clients"))
                    .size(Self::TITLE_TEXT_SIZE)
                    .align_y(Alignment::Center),
            )
            .push(Space::new(Length::Fill, Length::Shrink));

        if self.current_screen == ClientsScreen::AddEdit
            && self
                .add_edit_client
                .as_ref()
                .is_some_and(|x| x.id.is_some())
        {
            result_row = result_row.push(delete_button);
        }

        result_row = result_row
            .push(add_cancel_button)
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .spacing(spacing);

        result_row.into()
    }

    /// Returns the view of the clients grid
    fn view_clients_grid(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        if self.clients.is_empty() {
            return container(text(fl!("no-clients")).size(Self::TITLE_TEXT_SIZE))
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .padding(50.)
                .into();
        }

        let title_row = Row::new()
            .push(
                text(fl!("name"))
                    .size(Self::TITLE_TEXT_SIZE)
                    .width(250.)
                    .align_y(Alignment::Center),
            )
            .push(
                text("TD")
                    .size(Self::TITLE_TEXT_SIZE)
                    .width(100.)
                    .align_y(Alignment::Center),
            )
            .push(
                text(fl!("identity-document"))
                    .size(Self::TITLE_TEXT_SIZE)
                    .width(200.)
                    .align_y(Alignment::Center),
            )
            .push(
                text(fl!("country"))
                    .size(Self::TITLE_TEXT_SIZE)
                    .width(150.)
                    .align_y(Alignment::Center),
            )
            .push(match self.page_mode {
                ClientsPageMode::Normal => text(fl!("edit"))
                    .size(Self::TITLE_TEXT_SIZE)
                    .width(150.)
                    .align_y(Alignment::Center)
                    .align_x(Alignment::End),
                ClientsPageMode::Selection => text(fl!("select"))
                    .size(Self::TITLE_TEXT_SIZE)
                    .width(Length::Fixed(150.))
                    .align_y(Alignment::Center)
                    .align_x(Alignment::End),
            })
            .width(Length::Shrink)
            .align_y(Alignment::Center);

        // Calculate the indices for the current page
        let start_index: usize = self.clients_pagination_state.current_page as usize
            * self.clients_pagination_state.items_per_page as usize;
        let end_index = usize::min(
            start_index + self.clients_pagination_state.items_per_page as usize,
            self.clients.len(),
        );

        let mut grid = Column::new()
            .push(title_row)
            .align_x(Alignment::Center)
            .spacing(spacing)
            .width(Length::Shrink);

        for client in &self.clients[start_index..end_index] {
            let row = Row::new()
                .width(Length::Shrink)
                .push(
                    text(format!(
                        "{} {} {}",
                        &client.name, &client.first_surname, &client.second_surname
                    ))
                    .size(Self::TEXT_SIZE)
                    .width(250.)
                    .align_y(Alignment::Center),
                )
                .push(
                    text(&*client.identity_document_type_name)
                        .size(Self::TEXT_SIZE)
                        .width(100.)
                        .align_y(Alignment::Center),
                )
                .push(
                    text(&client.identity_document)
                        .size(Self::TEXT_SIZE)
                        .width(200.)
                        .align_y(Alignment::Center),
                )
                .push(
                    text(&client.country)
                        .size(Self::TEXT_SIZE)
                        .width(150.)
                        .align_y(Alignment::Center),
                )
                .push(
                    container(match self.page_mode {
                        ClientsPageMode::Normal => button(
                            text(fl!("edit"))
                                .size(Self::TEXT_SIZE)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::AskEditClient(client.id.unwrap()))
                        .width(Length::Shrink),
                        ClientsPageMode::Selection => button(
                            text(fl!("select"))
                                .size(Self::TEXT_SIZE)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::AskSelectClient(client.id.unwrap()))
                        .width(Length::Shrink),
                    })
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
            &self.clients_pagination_state.current_page + 1
        )));
        grid = grid.push(Space::with_height(Length::Fill));
        grid = grid.push(
            Row::new()
                .width(850.)
                .push(
                    button(
                        text(fl!("back"))
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                            .width(Length::Fill)
                            .height(button_height),
                    )
                    .on_press(Message::ClientsPaginationAction(PaginationAction::Back)),
                )
                .push(
                    button(
                        text(fl!("next"))
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                            .width(Length::Fill)
                            .height(button_height),
                    )
                    .on_press(Message::ClientsPaginationAction(PaginationAction::Forward)),
                )
                .spacing(spacing),
        );

        grid.into()
    }

    /// Returns the view of the room types add/edit screen
    fn view_add_edit_screen(&self) -> Element<Message> {
        if let Some(client) = &self.add_edit_client {
            let spacing = Pixels::from(Self::GLOBAL_SPACING);

            // First Name
            let name_label = text(format!("{}*", fl!("name"))).width(Length::Fill);
            let name_input = text_input(fl!("name").as_str(), &client.name)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Name))
                .size(Self::TEXT_SIZE)
                .width(Length::Fill);

            // First Surname
            let first_surname_label = text(fl!("first-surname")).width(Length::Fill);
            let first_surname_input =
                text_input(fl!("first-surname").as_str(), &client.first_surname)
                    .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::FirstSurname))
                    .size(Self::TEXT_SIZE)
                    .width(Length::Fill);

            // Second Surname
            let second_surname_label = text(fl!("second-surname")).width(Length::Fill);
            let second_surname_input =
                text_input(fl!("second-surname").as_str(), &client.second_surname)
                    .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::SecondSurname))
                    .size(Self::TEXT_SIZE)
                    .width(Length::Fill);

            // Gender
            let gender_label = text(fl!("gender")).width(Length::Fill);
            let selected_gender = self.genders.iter().find(|g| g.id == client.gender_id);
            let gender_selector = pick_list(self.genders.clone(), selected_gender, |g| {
                Message::UpdatedSelectedGenderId(g.id.unwrap_or_default())
            })
            .width(Length::Fill);

            //  BirthDate
            let birthdate_date_label =
                text(format!("{} (yyyy-mm-dd)", fl!("birthdate"))).width(Length::Fill);
            let birthdate_iced_aw_date = if let Some(db_date) = &client.birthdate {
                Date {
                    year: db_date.year(),
                    month: db_date.month(),
                    day: db_date.day(),
                }
            } else {
                Date::today()
            };
            let birthdate_date_picker = DatePicker::new(
                self.show_birthdate_date_picker,
                birthdate_iced_aw_date,
                button(text(fl!("edit")))
                    .on_press(Message::ShowDatePicker(ClientDateInputFields::Birthdate)),
                Message::CancelDateOperation,
                |date| Message::UpdateDateField(date, ClientDateInputFields::Birthdate),
            );
            let birthdate_input = text_input(fl!("birthdate").as_str(), &client.birthdate_string)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Birthdate))
                .size(Self::TEXT_SIZE)
                .width(Length::Fill);

            let birthdate_input_row = row![birthdate_input, birthdate_date_picker]
                .align_y(Alignment::Center)
                .spacing(spacing);

            // Identity Document
            let identity_document_label =
                text(format!("{}*", fl!("identity-document"))).width(Length::Fill);
            let identity_document_input =
                text_input(fl!("identity-document").as_str(), &client.identity_document)
                    .on_input(|c| {
                        Message::TextInputUpdate(c, ClientTextInputFields::IdentityDocument)
                    })
                    .size(Self::TEXT_SIZE)
                    .width(Length::Fill);

            // Identity Document Type
            let identity_document_type_label =
                text(format!("{}*", fl!("identity-document-type"))).width(Length::Fill);
            let selected_identity_document_type = self
                .identity_document_types
                .iter()
                .find(|it| it.id == client.identity_document_type_id);
            let identity_document_type_selector = pick_list(
                self.identity_document_types.clone(),
                selected_identity_document_type,
                |idt| Message::UpdatedSelectedDocumentTypeId(idt.id.unwrap_or_default()),
            )
            .width(Length::Fill);

            // Identity Document Expedition Date
            let identity_document_expedition_date_label = text(format!(
                "{} (yyyy-mm-dd)",
                fl!("identity-document-expedition-date")
            ))
            .width(Length::Fill);
            let expedition_iced_aw_date =
                if let Some(db_date) = &client.identity_document_expedition_date {
                    Date {
                        year: db_date.year(),
                        month: db_date.month(),
                        day: db_date.day(),
                    }
                } else {
                    Date::today()
                };
            let identity_document_expedition_date_picker = DatePicker::new(
                self.show_expedition_date_picker,
                expedition_iced_aw_date,
                button(text(fl!("edit"))).on_press(Message::ShowDatePicker(
                    ClientDateInputFields::IdentityDocumentExpeditionDate,
                )),
                Message::CancelDateOperation,
                |date| {
                    Message::UpdateDateField(
                        date,
                        ClientDateInputFields::IdentityDocumentExpeditionDate,
                    )
                },
            );
            let identity_document_expedition_date_input = text_input(
                fl!("identity-document-expedition-date").as_str(),
                &client.identity_document_expedition_date_string,
            )
            .on_input(|c| {
                Message::TextInputUpdate(c, ClientTextInputFields::IdentityDocumentExpeditionDate)
            })
            .size(Self::TEXT_SIZE)
            .width(Length::Fill);
            let identity_document_expedition_date_input_row = row![
                identity_document_expedition_date_input,
                identity_document_expedition_date_picker
            ]
            .align_y(Alignment::Center)
            .spacing(spacing);

            // Identity Document Expiration Date
            let identity_document_expiration_date_label = text(format!(
                "{} (yyyy-mm-dd)",
                fl!("identity-document-expiration-date")
            ))
            .width(Length::Fill);
            let expiration_iced_aw_date =
                if let Some(db_date) = &client.identity_document_expiration_date {
                    Date {
                        year: db_date.year(),
                        month: db_date.month(),
                        day: db_date.day(),
                    }
                } else {
                    Date::today()
                };
            let identity_document_expiration_date_picker = DatePicker::new(
                self.show_expiration_date_picker,
                expiration_iced_aw_date,
                button(text(fl!("edit"))).on_press(Message::ShowDatePicker(
                    ClientDateInputFields::IdentityDocumentExpirationDate,
                )),
                Message::CancelDateOperation,
                |date| {
                    Message::UpdateDateField(
                        date,
                        ClientDateInputFields::IdentityDocumentExpirationDate,
                    )
                },
            );
            let identity_document_expiration_date_input = text_input(
                fl!("identity-document-expiration-date").as_str(),
                &client.identity_document_expiration_date_string,
            )
            .on_input(|c| {
                Message::TextInputUpdate(c, ClientTextInputFields::IdentityDocumentExpirationDate)
            })
            .size(Self::TEXT_SIZE)
            .width(Length::Fill);
            let identity_document_expiration_date_input_row = row![
                identity_document_expiration_date_input,
                identity_document_expiration_date_picker
            ]
            .align_y(Alignment::Center)
            .spacing(spacing);

            // Address
            let address_label = text(fl!("address")).width(Length::Fill);
            let address_input = text_input(fl!("address").as_str(), &client.address)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Address))
                .size(Self::TEXT_SIZE)
                .width(Length::Fill);

            // Postal Code
            let postal_code_label = text(fl!("postal-code")).width(Length::Fill);
            let postal_code_input = text_input(fl!("postal-code").as_str(), &client.postal_code)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::PostalCode))
                .size(Self::TEXT_SIZE)
                .width(Length::Fill);

            // City
            let city_label = text(fl!("city")).width(Length::Fill);
            let city_input = text_input(fl!("city").as_str(), &client.city)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::City))
                .size(Self::TEXT_SIZE)
                .width(Length::Fill);

            // Province
            let province_label = text(fl!("province")).width(Length::Fill);
            let province_input = text_input(fl!("province").as_str(), &client.province)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Province))
                .size(Self::TEXT_SIZE)
                .width(Length::Fill);

            // Country
            let country_label = text(format!("{}*", fl!("country"))).width(Length::Fill);
            let country_input = text_input(fl!("country").as_str(), &client.country)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Country))
                .size(Self::TEXT_SIZE)
                .width(Length::Fill);

            // Nationality
            let nationality_label = text(fl!("nationality")).width(Length::Fill);
            let nationality_input = text_input(fl!("nationality").as_str(), &client.nationality)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Nationality))
                .size(Self::TEXT_SIZE)
                .width(Length::Fill);

            // Phone Number
            let phone_number_label = text(fl!("phone-number")).width(Length::Fill);
            let phone_number_input = text_input(fl!("phone-number").as_str(), &client.phone_number)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::PhoneNumber))
                .size(Self::TEXT_SIZE)
                .width(Length::Fill);

            // Mobile Phone
            let mobile_phone_label = text(fl!("mobile-phone")).width(Length::Fill);
            let mobile_phone_input = text_input(fl!("mobile-phone").as_str(), &client.mobile_phone)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::MobilePhone))
                .size(Self::TEXT_SIZE)
                .width(Length::Fill);

            // Submit
            let submit_button = if client.id.is_some() {
                button(
                    text(fl!("edit"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(Self::TEXT_SIZE),
                )
                .on_press(Message::EditCurrentClient)
                .width(Length::Fill)
            } else {
                button(
                    text(fl!("add"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(Self::TEXT_SIZE),
                )
                .on_press(Message::AddCurrentClient)
                .width(Length::Fill)
            };

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
                .push(birthdate_input_row)
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
                .push(identity_document_expedition_date_input_row)
                .width(Length::Fill)
                .spacing(1.);

            let identity_document_expiration_date_column = Column::new()
                .push(identity_document_expiration_date_label)
                .push(identity_document_expiration_date_input_row)
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
                        .spacing(spacing)
                        .width(850.),
                )
                .push(
                    Row::new()
                        .push(gender_input_column)
                        .push(birthdate_input_column)
                        .spacing(spacing)
                        .width(850.),
                )
                .push(
                    Row::new()
                        .push(identity_document_column)
                        .push(identity_document_type_input_column)
                        .spacing(spacing)
                        .width(850.),
                )
                .push(
                    Row::new()
                        .push(identity_document_expedition_date_column)
                        .push(identity_document_expiration_date_column)
                        .spacing(spacing)
                        .width(850.),
                )
                .push(address_input_column)
                .push(
                    Row::new()
                        .push(postal_code_input_column)
                        .push(city_input_column)
                        .push(province_input_column)
                        .spacing(spacing)
                        .width(850.),
                )
                .push(
                    Row::new()
                        .push(country_input_column)
                        .push(nationality_input_column)
                        .spacing(spacing)
                        .width(850.),
                )
                .push(
                    Row::new()
                        .push(phone_number_input_column)
                        .push(mobile_phone_input_column)
                        .spacing(spacing)
                        .width(850.),
                )
                .push(submit_button)
                .width(850.)
                .spacing(spacing);

            container(form_column)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .padding(50.)
                .into()
        } else {
            container(text("Error"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .padding(50.)
                .into()
        }
    }

    //
    //  END OF VIEW COMPOSING
    //
}

#[allow(clippy::if_same_then_else)]
fn is_client_valid(client: &Client) -> bool {
    if client.identity_document_type_id.is_none() {
        return false;
    } else if client.identity_document.is_empty() {
        return false;
    } else if client.name.is_empty() {
        return false;
    } else if client.country.is_empty() {
        return false;
    }

    if !client.birthdate_string.is_empty() {
        let v = check_date_format(&client.birthdate_string);
        return v;
    }

    if !client.identity_document_expedition_date_string.is_empty() {
        let v = check_date_format(&client.identity_document_expedition_date_string);
        return v;
    }

    if !client.identity_document_expiration_date_string.is_empty() {
        let v = check_date_format(&client.identity_document_expiration_date_string);
        return v;
    }

    true
}
