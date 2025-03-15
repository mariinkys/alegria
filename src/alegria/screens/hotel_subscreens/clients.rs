// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use iced::{
    Alignment, Element, Length, Padding, Pixels, Task,
    widget::{self, Row, Space},
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
    },
    fl,
};

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
}

#[derive(Debug, Clone)]
pub enum ClientDateInputFields {
    IdentityDocumentExpeditionDate,
    IdentityDocumentExpirationDate,
    Birthdate,
}

pub struct Clients {
    /// Database of the application
    pub database: Option<Arc<PgPool>>,
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
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back

    FetchClients,            // Fetches all the current clients
    SetClients(Vec<Client>), // Sets the clients on the app state

    InitPage, // Intended to be called from Hotel when first opening the page, asks for the necessary data and executes the appropiate callbacks
    SetIdentityDocumentTypes(Vec<IdentityDocumentType>), // Sets the identity document types on the app state
    SetGenders(Vec<Gender>), // Sets the identity document types on the app state

    SetAddEditClient(Client), // changes the screen and the add_edit_client state (after asking for an edit Client)
    AskEditClient(i32), // Callback after asking to edit a client, searches the client on the db
    AskAddClient, // Callback after asking to add a room, changes the screen and the add_edit_client state
    CancelClientOperation, // Callback after asking to cancel an add or an edit

    TextInputUpdate(String, ClientTextInputFields), // Callback when using the text inputs to add or edit a client
    UpdatedSelectedDocumentTypeId(i32), // Callback after selecting a new DocumentTypeId for the current client
    UpdatedSelectedGenderId(i32), // Callback after selecting a new GenderId for the current client
    ShowDatePicker(ClientDateInputFields),
    CancelDateOperation,
    UpdateDateField(date_picker::Date, ClientDateInputFields),

    AddCurrentClient,      // Tries to Add the current client to the database
    EditCurrentClient,     // Tries to Edit the current client on the database
    DeleteCurrentClient,   // Tries to delete the current Client
    ModifiedCurrentClient, // Callback after delete/update/add of a current Client
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum ClientsInstruction {
    Back, // Asks the parent (app.rs) to go back
}

impl Clients {
    /// Initializes the screen
    pub fn init() -> Self {
        Self {
            database: None,
            current_screen: ClientsScreen::List,
            clients: Vec::new(),
            identity_document_types: Vec::new(),
            genders: Vec::new(),
            add_edit_client: None,
            show_expedition_date_picker: false,
            show_expiration_date_picker: false,
            show_birthdate_date_picker: false,
        }
    }

    /// Cleans the state of the screen preserving the database
    /// intended to be called when switching to another screen in order to save memory.
    pub fn clean_state(database: Option<Arc<PgPool>>) -> Self {
        Self {
            database,
            current_screen: ClientsScreen::List,
            clients: Vec::new(),
            identity_document_types: Vec::new(),
            genders: Vec::new(),
            add_edit_client: None,
            show_expedition_date_picker: false,
            show_expiration_date_picker: false,
            show_birthdate_date_picker: false,
        }
    }

    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<ClientsInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            Message::Back => action.add_instruction(ClientsInstruction::Back),

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
                self.add_edit_client = Some(client);
                self.current_screen = ClientsScreen::AddEdit;
            }
            // Callback after asking to edit a client, searches the client on the db
            Message::AskEditClient(client_id) => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        Client::get_single(pool.clone(), client_id),
                        |res| match res {
                            Ok(res) => Message::SetAddEditClient(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetAddEditClient(Client::default())
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
            Message::CancelDateOperation => {
                self.show_expedition_date_picker = false;
                self.show_expiration_date_picker = false;
                self.show_birthdate_date_picker = false;
            }
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
                                }
                                ClientDateInputFields::IdentityDocumentExpirationDate => {
                                    client.identity_document_expiration_date =
                                        Some(NaiveDateTime::new(date, NaiveTime::MIN));
                                }
                                ClientDateInputFields::Birthdate => {
                                    client.birthdate =
                                        Some(NaiveDateTime::new(date, NaiveTime::MIN));
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
                if let Some(client) = &self.add_edit_client {
                    // TODO: Proper validation
                    if is_client_valid(client) && client.id.is_none() {
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
                if let Some(client) = &self.add_edit_client {
                    // TODO: Proper validation
                    if is_client_valid(client) && client.id.is_some() {
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
            // Callback after add/update/delete of the current Room
            Message::ModifiedCurrentClient => {
                self.add_edit_client = None;
                self.current_screen = ClientsScreen::List;
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
            ClientsScreen::List => self.view_clients_grid(),
            ClientsScreen::AddEdit => self.view_add_edit_screen(),
        };

        widget::Column::new()
            .push(header_row)
            .push(content)
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

        let back_button = widget::Button::new(
            widget::Text::new(fl!("back"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .on_press(Message::Back)
        .height(button_height);

        let add_cancel_button = match &self.current_screen {
            ClientsScreen::List => widget::Button::new(
                widget::Text::new(fl!("add"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::AskAddClient)
            .height(button_height),
            ClientsScreen::AddEdit => widget::Button::new(
                widget::Text::new(fl!("cancel"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .style(widget::button::danger)
            .on_press(Message::CancelClientOperation)
            .height(button_height),
        };

        let delete_button = widget::Button::new(
            widget::Text::new(fl!("delete"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .style(widget::button::secondary)
        .on_press(Message::DeleteCurrentClient)
        .height(button_height);

        let mut result_row = widget::Row::new();
        if self.current_screen == ClientsScreen::List {
            result_row = result_row.push(back_button);
        }

        result_row = result_row
            .push(
                widget::Text::new(fl!("clients"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
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

        if self.clients.is_empty() {
            return widget::Container::new(
                widget::Text::new(fl!("no-clients")).size(Pixels::from(Self::TITLE_TEXT_SIZE)),
            )
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding(Padding::new(50.))
            .into();
        }

        let title_row = widget::Row::new()
            .push(
                widget::Text::new(fl!("name"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .width(Length::Fixed(250.))
                    .align_y(Alignment::Center),
            )
            .push(
                widget::Text::new("TD")
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .width(Length::Fixed(100.))
                    .align_y(Alignment::Center),
            )
            .push(
                widget::Text::new(fl!("identity-document"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .width(Length::Fixed(200.))
                    .align_y(Alignment::Center),
            )
            .push(
                widget::Text::new(fl!("country"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .width(Length::Fixed(150.))
                    .align_y(Alignment::Center),
            )
            .push(
                widget::Text::new(fl!("edit"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .width(Length::Fixed(150.))
                    .align_y(Alignment::Center)
                    .align_x(Alignment::End),
            )
            .width(Length::Shrink)
            .align_y(Alignment::Center);

        let mut grid = widget::Column::new()
            .push(title_row)
            .align_x(Alignment::Center)
            .spacing(spacing)
            .width(Length::Shrink);

        for client in &self.clients {
            let row = widget::Row::new()
                .width(Length::Shrink)
                .push(
                    widget::Text::new(format!(
                        "{} {} {}",
                        &client.name, &client.first_surname, &client.second_surname
                    ))
                    .size(Pixels::from(Self::TEXT_SIZE))
                    .width(Length::Fixed(250.))
                    .align_y(Alignment::Center),
                )
                .push(
                    widget::Text::new(&client.identity_document_type_name)
                        .size(Pixels::from(Self::TEXT_SIZE))
                        .width(Length::Fixed(100.))
                        .align_y(Alignment::Center),
                )
                .push(
                    widget::Text::new(&client.identity_document)
                        .size(Pixels::from(Self::TEXT_SIZE))
                        .width(Length::Fixed(200.))
                        .align_y(Alignment::Center),
                )
                .push(
                    widget::Text::new(&client.country)
                        .size(Pixels::from(Self::TEXT_SIZE))
                        .width(Length::Fixed(150.))
                        .align_y(Alignment::Center),
                )
                .push(
                    widget::Container::new(
                        widget::Button::new(
                            widget::Text::new(fl!("edit"))
                                .size(Pixels::from(Self::TEXT_SIZE))
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::AskEditClient(client.id.unwrap()))
                        .width(Length::Shrink),
                    )
                    .width(Length::Fixed(150.))
                    .align_x(Alignment::End)
                    .align_y(Alignment::Center),
                )
                .align_y(Alignment::Center);

            // Limit Rule size to sum of all column widths
            grid = grid.push(
                widget::Row::new()
                    .width(Length::Fixed(850.))
                    .push(widget::Rule::horizontal(Pixels::from(1.))),
            );
            grid = grid.push(row);
        }

        grid = grid.push(
            widget::Row::new()
                .width(Length::Fixed(850.))
                .push(widget::Rule::horizontal(Pixels::from(1.))),
        );
        widget::Container::new(grid)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding(Padding::new(50.))
            .into()
    }

    /// Returns the view of the room types add/edit screen
    fn view_add_edit_screen(&self) -> Element<Message> {
        if let Some(client) = &self.add_edit_client {
            let spacing = Pixels::from(Self::GLOBAL_SPACING);

            // First Name
            let name_label = widget::Text::new(format!("{}*", fl!("name"))).width(Length::Fill);
            let name_input = widget::TextInput::new(fl!("name").as_str(), &client.name)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Name))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);

            // First Surname
            let first_surname_label = widget::Text::new(fl!("first-surname")).width(Length::Fill);
            let first_surname_input =
                widget::TextInput::new(fl!("first-surname").as_str(), &client.first_surname)
                    .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::FirstSurname))
                    .size(Pixels::from(Self::TEXT_SIZE))
                    .width(Length::Fill);

            // Second Surname
            let second_surname_label = widget::Text::new(fl!("second-surname")).width(Length::Fill);
            let second_surname_input =
                widget::TextInput::new(fl!("second-surname").as_str(), &client.second_surname)
                    .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::SecondSurname))
                    .size(Pixels::from(Self::TEXT_SIZE))
                    .width(Length::Fill);

            // Gender
            let gender_label = widget::Text::new(fl!("gender")).width(Length::Fill);
            let selected_gender = self.genders.iter().find(|g| g.id == client.gender_id);
            let gender_selector =
                widget::PickList::new(self.genders.clone(), selected_gender, |g| {
                    Message::UpdatedSelectedGenderId(g.id.unwrap_or_default())
                })
                .width(Length::Fill);

            //  BirthDate
            let birthdate_date_label =
                widget::Text::new(format!("{} (yyyy-mm-dd)", fl!("birthdate"))).width(Length::Fill);
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
                widget::Button::new(widget::Text::new(fl!("edit")))
                    .on_press(Message::ShowDatePicker(ClientDateInputFields::Birthdate)),
                Message::CancelDateOperation,
                |date| Message::UpdateDateField(date, ClientDateInputFields::Birthdate),
            );
            let birthdate_input = match &client.birthdate {
                Some(_) => widget::TextInput::new(
                    fl!("birthdate").as_str(),
                    &birthdate_iced_aw_date.to_string(),
                )
                .style(|t, _| widget::text_input::default(t, widget::text_input::Status::Active))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill),
                None => widget::TextInput::new(fl!("birthdate").as_str(), "")
                    .style(|t, _| {
                        widget::text_input::default(t, widget::text_input::Status::Active)
                    })
                    .size(Pixels::from(Self::TEXT_SIZE))
                    .width(Length::Fill),
            };
            let birthdate_input_row = widget::Row::new()
                .push(birthdate_input)
                .push(birthdate_date_picker)
                .align_y(Alignment::Center)
                .spacing(spacing);

            // Identity Document
            let identity_document_label =
                widget::Text::new(format!("{}*", fl!("identity-document"))).width(Length::Fill);
            let identity_document_input = widget::TextInput::new(
                fl!("identity-document").as_str(),
                &client.identity_document,
            )
            .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::IdentityDocument))
            .size(Pixels::from(Self::TEXT_SIZE))
            .width(Length::Fill);

            // Identity Document Type
            let identity_document_type_label =
                widget::Text::new(format!("{}*", fl!("identity-document-type")))
                    .width(Length::Fill);
            let selected_identity_document_type = self
                .identity_document_types
                .iter()
                .find(|it| it.id == client.identity_document_type_id);
            let identity_document_type_selector = widget::PickList::new(
                self.identity_document_types.clone(),
                selected_identity_document_type,
                |idt| Message::UpdatedSelectedDocumentTypeId(idt.id.unwrap_or_default()),
            )
            .width(Length::Fill);

            // Identity Document Expedition Date
            let identity_document_expedition_date_label = widget::Text::new(format!(
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
                widget::Button::new(widget::Text::new(fl!("edit"))).on_press(
                    Message::ShowDatePicker(ClientDateInputFields::IdentityDocumentExpeditionDate),
                ),
                Message::CancelDateOperation,
                |date| {
                    Message::UpdateDateField(
                        date,
                        ClientDateInputFields::IdentityDocumentExpeditionDate,
                    )
                },
            );
            let identity_document_expedition_date_input = match &client
                .identity_document_expedition_date
            {
                Some(_) => widget::TextInput::new(
                    fl!("identity-document-expedition-date").as_str(),
                    &expedition_iced_aw_date.to_string(),
                )
                .style(|t, _| widget::text_input::default(t, widget::text_input::Status::Active))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill),
                None => {
                    widget::TextInput::new(fl!("identity-document-expedition-date").as_str(), "")
                        .style(|t, _| {
                            widget::text_input::default(t, widget::text_input::Status::Active)
                        })
                        .size(Pixels::from(Self::TEXT_SIZE))
                        .width(Length::Fill)
                }
            };
            let identity_document_expedition_date_input_row = widget::Row::new()
                .push(identity_document_expedition_date_input)
                .push(identity_document_expedition_date_picker)
                .align_y(Alignment::Center)
                .spacing(spacing);

            // Identity Document Expiration Date
            let identity_document_expiration_date_label = widget::Text::new(format!(
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
                widget::Button::new(widget::Text::new(fl!("edit"))).on_press(
                    Message::ShowDatePicker(ClientDateInputFields::IdentityDocumentExpirationDate),
                ),
                Message::CancelDateOperation,
                |date| {
                    Message::UpdateDateField(
                        date,
                        ClientDateInputFields::IdentityDocumentExpirationDate,
                    )
                },
            );
            let identity_document_expiration_date_input = match &client
                .identity_document_expiration_date
            {
                Some(_) => widget::TextInput::new(
                    fl!("identity-document-expiration-date").as_str(),
                    &expiration_iced_aw_date.to_string(),
                )
                .style(|t, _| widget::text_input::default(t, widget::text_input::Status::Active))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill),
                None => {
                    widget::TextInput::new(fl!("identity-document-expiration-date").as_str(), "")
                        .style(|t, _| {
                            widget::text_input::default(t, widget::text_input::Status::Active)
                        })
                        .size(Pixels::from(Self::TEXT_SIZE))
                        .width(Length::Fill)
                }
            };
            let identity_document_expiration_date_input_row = widget::Row::new()
                .push(identity_document_expiration_date_input)
                .push(identity_document_expiration_date_picker)
                .align_y(Alignment::Center)
                .spacing(spacing);

            // Address
            let address_label = widget::Text::new(fl!("address")).width(Length::Fill);
            let address_input = widget::TextInput::new(fl!("address").as_str(), &client.address)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Address))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);

            // Postal Code
            let postal_code_label = widget::Text::new(fl!("postal-code")).width(Length::Fill);
            let postal_code_input =
                widget::TextInput::new(fl!("postal-code").as_str(), &client.postal_code)
                    .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::PostalCode))
                    .size(Pixels::from(Self::TEXT_SIZE))
                    .width(Length::Fill);

            // City
            let city_label = widget::Text::new(fl!("city")).width(Length::Fill);
            let city_input = widget::TextInput::new(fl!("city").as_str(), &client.city)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::City))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);

            // Province
            let province_label = widget::Text::new(fl!("province")).width(Length::Fill);
            let province_input = widget::TextInput::new(fl!("province").as_str(), &client.province)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Province))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);

            // Country
            let country_label =
                widget::Text::new(format!("{}*", fl!("country"))).width(Length::Fill);
            let country_input = widget::TextInput::new(fl!("country").as_str(), &client.country)
                .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Country))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);

            // Nationality
            let nationality_label = widget::Text::new(fl!("nationality")).width(Length::Fill);
            let nationality_input =
                widget::TextInput::new(fl!("nationality").as_str(), &client.nationality)
                    .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::Nationality))
                    .size(Pixels::from(Self::TEXT_SIZE))
                    .width(Length::Fill);

            // Phone Number
            let phone_number_label = widget::Text::new(fl!("phone-number")).width(Length::Fill);
            let phone_number_input =
                widget::TextInput::new(fl!("phone-number").as_str(), &client.phone_number)
                    .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::PhoneNumber))
                    .size(Pixels::from(Self::TEXT_SIZE))
                    .width(Length::Fill);

            // Mobile Phone
            let mobile_phone_label = widget::Text::new(fl!("mobile-phone")).width(Length::Fill);
            let mobile_phone_input =
                widget::TextInput::new(fl!("mobile-phone").as_str(), &client.mobile_phone)
                    .on_input(|c| Message::TextInputUpdate(c, ClientTextInputFields::MobilePhone))
                    .size(Pixels::from(Self::TEXT_SIZE))
                    .width(Length::Fill);

            // Submit
            let submit_button = if client.id.is_some() {
                widget::Button::new(
                    widget::Text::new(fl!("edit"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(Pixels::from(Self::TEXT_SIZE)),
                )
                .on_press(Message::EditCurrentClient)
                .width(Length::Fill)
            } else {
                widget::Button::new(
                    widget::Text::new(fl!("add"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(Pixels::from(Self::TEXT_SIZE)),
                )
                .on_press(Message::AddCurrentClient)
                .width(Length::Fill)
            };

            let name_input_column = widget::Column::new()
                .push(name_label)
                .push(name_input)
                .width(Length::Fill)
                .spacing(1.);

            let first_surname_input_column = widget::Column::new()
                .push(first_surname_label)
                .push(first_surname_input)
                .width(Length::Fill)
                .spacing(1.);

            let second_surname_input_column = widget::Column::new()
                .push(second_surname_label)
                .push(second_surname_input)
                .width(Length::Fill)
                .spacing(1.);

            let gender_input_column = widget::Column::new()
                .push(gender_label)
                .push(gender_selector)
                .width(Length::Fill)
                .spacing(1.);

            let birthdate_input_column = widget::Column::new()
                .push(birthdate_date_label)
                .push(birthdate_input_row)
                .width(Length::Fill)
                .spacing(1.);

            let identity_document_column = widget::Column::new()
                .push(identity_document_label)
                .push(identity_document_input)
                .width(Length::Fill)
                .spacing(1.);

            let identity_document_type_input_column = widget::Column::new()
                .push(identity_document_type_label)
                .push(identity_document_type_selector)
                .width(Length::Fill)
                .spacing(1.);

            let identity_document_expedition_date_column = widget::Column::new()
                .push(identity_document_expedition_date_label)
                .push(identity_document_expedition_date_input_row)
                .width(Length::Fill)
                .spacing(1.);

            let identity_document_expiration_date_column = widget::Column::new()
                .push(identity_document_expiration_date_label)
                .push(identity_document_expiration_date_input_row)
                .width(Length::Fill)
                .spacing(1.);

            let address_input_column = widget::Column::new()
                .push(address_label)
                .push(address_input)
                .width(Length::Fill)
                .spacing(1.);

            let postal_code_input_column = widget::Column::new()
                .push(postal_code_label)
                .push(postal_code_input)
                .width(Length::Fill)
                .spacing(1.);

            let city_input_column = widget::Column::new()
                .push(city_label)
                .push(city_input)
                .width(Length::Fill)
                .spacing(1.);

            let province_input_column = widget::Column::new()
                .push(province_label)
                .push(province_input)
                .width(Length::Fill)
                .spacing(1.);

            let country_input_column = widget::Column::new()
                .push(country_label)
                .push(country_input)
                .width(Length::Fill)
                .spacing(1.);

            let nationality_input_column = widget::Column::new()
                .push(nationality_label)
                .push(nationality_input)
                .width(Length::Fill)
                .spacing(1.);

            let phone_number_input_column = widget::Column::new()
                .push(phone_number_label)
                .push(phone_number_input)
                .width(Length::Fill)
                .spacing(1.);

            let mobile_phone_input_column = widget::Column::new()
                .push(mobile_phone_label)
                .push(mobile_phone_input)
                .width(Length::Fill)
                .spacing(1.);

            let form_column = widget::Column::new()
                .push(name_input_column)
                .push(
                    Row::new()
                        .push(first_surname_input_column)
                        .push(second_surname_input_column)
                        .spacing(spacing)
                        .width(Length::Fixed(850.)),
                )
                .push(
                    Row::new()
                        .push(gender_input_column)
                        .push(birthdate_input_column)
                        .spacing(spacing)
                        .width(Length::Fixed(850.)),
                )
                .push(
                    Row::new()
                        .push(identity_document_column)
                        .push(identity_document_type_input_column)
                        .spacing(spacing)
                        .width(Length::Fixed(850.)),
                )
                .push(
                    Row::new()
                        .push(identity_document_expedition_date_column)
                        .push(identity_document_expiration_date_column)
                        .spacing(spacing)
                        .width(Length::Fixed(850.)),
                )
                .push(address_input_column)
                .push(
                    Row::new()
                        .push(postal_code_input_column)
                        .push(city_input_column)
                        .push(province_input_column)
                        .spacing(spacing)
                        .width(Length::Fixed(850.)),
                )
                .push(
                    Row::new()
                        .push(country_input_column)
                        .push(nationality_input_column)
                        .spacing(spacing)
                        .width(Length::Fixed(850.)),
                )
                .push(
                    Row::new()
                        .push(phone_number_input_column)
                        .push(mobile_phone_input_column)
                        .spacing(spacing)
                        .width(Length::Fixed(850.)),
                )
                .push(submit_button)
                .width(Length::Fixed(850.))
                .spacing(spacing);

            widget::Container::new(form_column)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .padding(Padding::new(50.))
                .into()
        } else {
            widget::Container::new(widget::Text::new("Error"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .padding(Padding::new(50.))
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
    true
}
