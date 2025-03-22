use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use iced::{
    Alignment, Element, Length, Padding, Pixels, Task,
    widget::{self},
};
use iced_aw::{
    DatePicker,
    date_picker::{self, Date},
};
use sqlx::PgPool;

use crate::{
    alegria::{
        action::AlegriaAction,
        core::models::{reservation::Reservation, room::Room, sold_room::SoldRoom},
        screens::hotel::clients::{self, Clients, ClientsPageMode},
    },
    fl,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AddReservationPageMode {
    AddingReservation,
    PickingClient,
}

#[derive(Debug, Clone)]
pub enum ReservationDateInputFields {
    EntryDate,
    DepartureDate,
}

#[derive(Default, Debug, Clone)]
struct ReservationDateInputState {
    show_entry_date_picker: bool,
    show_departure_date_picker: bool,
}

pub struct AddReservationPage {
    /// Database of the application (needed for the client selector)
    pub database: Option<Arc<PgPool>>,
    /// Holds the state of the currently adding/editing reservation
    new_reservation: Option<Reservation>,
    /// Holds the state of the current page mode
    page_mode: AddReservationPageMode,
    /// Holds the state of the datepickers to input dates for a reservation
    new_reservation_datepickers_state: ReservationDateInputState,
    /// Holds the state of all the rooms (needed to create the kinda grid/calendar view)
    rooms: Arc<Vec<Room>>,
    /// Holds the state of all the reservations (of the given time/period)
    reservations: Arc<Vec<Reservation>>,
    /// Clients SubScreen of the reservation page (client selection)
    clients_selector: Clients,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back

    ShowDatePicker(ReservationDateInputFields), // Asks to open the requested date picker
    CancelDateOperation,                        // Cancels the datepicker changes
    UpdateDateField(date_picker::Date, ReservationDateInputFields), // Callback after submiting a new date via datepicker
    ToggleOccupiedCheckbox(bool), // Change occupied checkbox value for the current add reservation
    AddReservationRoom(i32, Option<f32>), // Asks to add a room to the vec of booked rooms of the current add reservation
    RemoveReservationRoom(i32), // Asks to remove a room to the vec of booked rooms of the current add reservation
    AddReservation,             // Tries to add the current reservation to the database

    OpenClientSelector,        // Asks to open the client selector page/component
    Clients(clients::Message), // Messages of the clients (selector) page
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum AddReservationsInstruction {
    Back,                           // Asks the parent (app.rs) to go back
    TryAddReservation(Reservation), // Asks the parent to add the reservation to the database
}

#[allow(clippy::derivable_impls)]
impl Default for AddReservationPage {
    fn default() -> Self {
        Self {
            database: None,
            new_reservation: None,
            page_mode: AddReservationPageMode::AddingReservation,
            new_reservation_datepickers_state: ReservationDateInputState::default(),
            rooms: Arc::default(),
            reservations: Arc::default(),
            clients_selector: Clients::init(),
        }
    }
}

impl AddReservationPage {
    /// Called when opening the add reservation page
    pub fn open_reservation(
        database: Option<Arc<PgPool>>,
        reservation: Reservation,
        rooms: Arc<Vec<Room>>,
        reservations: Arc<Vec<Reservation>>,
    ) -> Self {
        Self {
            database,
            new_reservation: Some(reservation),
            page_mode: AddReservationPageMode::AddingReservation,
            new_reservation_datepickers_state: ReservationDateInputState::default(),
            rooms,
            reservations,
            clients_selector: Clients::init(),
        }
    }

    /// Handles messages emitted by the application and its widgets.
    pub fn update(
        &mut self,
        message: Message,
    ) -> AlegriaAction<AddReservationsInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            Message::Back => match self.page_mode {
                AddReservationPageMode::AddingReservation => {
                    action.add_instruction(AddReservationsInstruction::Back);
                }
                AddReservationPageMode::PickingClient => {
                    self.clients_selector = Clients::init();
                    self.page_mode = AddReservationPageMode::AddingReservation;
                }
            },

            // Asks to open the requested date picker
            Message::ShowDatePicker(reservation_date_input_fields) => {
                match reservation_date_input_fields {
                    ReservationDateInputFields::EntryDate => {
                        self.new_reservation_datepickers_state
                            .show_entry_date_picker = true;
                    }
                    ReservationDateInputFields::DepartureDate => {
                        self.new_reservation_datepickers_state
                            .show_departure_date_picker = true;
                    }
                }
            }
            // Cancels the datepicker changes
            Message::CancelDateOperation => {
                self.new_reservation_datepickers_state
                    .show_entry_date_picker = false;
                self.new_reservation_datepickers_state
                    .show_departure_date_picker = false;
            }
            // Callback after submiting a new date via datepicker
            Message::UpdateDateField(iced_aw_date, field) => {
                let new_date = NaiveDate::from_ymd_opt(
                    iced_aw_date.year,
                    iced_aw_date.month,
                    iced_aw_date.day,
                );

                match new_date {
                    Some(date) => {
                        match field {
                            ReservationDateInputFields::EntryDate => {
                                if let Some(reservation) = self.new_reservation.as_mut() {
                                    if let Some(departure_date) = reservation.departure_date {
                                        if date < departure_date.date() {
                                            reservation.entry_date =
                                                Some(date.and_hms_opt(0, 0, 0).unwrap());
                                        }
                                    } else {
                                        reservation.entry_date =
                                            Some(date.and_hms_opt(0, 0, 0).unwrap());
                                    }
                                }
                            }
                            ReservationDateInputFields::DepartureDate => {
                                if let Some(reservation) = self.new_reservation.as_mut() {
                                    if let Some(entry_date) = reservation.entry_date {
                                        if date > entry_date.date() {
                                            reservation.departure_date =
                                                Some(date.and_hms_opt(0, 0, 0).unwrap());
                                        }
                                    } else {
                                        reservation.departure_date =
                                            Some(date.and_hms_opt(0, 0, 0).unwrap());
                                    }
                                }
                            }
                        }

                        self.new_reservation_datepickers_state
                            .show_entry_date_picker = false;
                        self.new_reservation_datepickers_state
                            .show_departure_date_picker = false;
                    }
                    None => {
                        eprintln!("Can't parse dates");
                    }
                }
            }
            // Change occupied checkbox value for the current reservation
            Message::ToggleOccupiedCheckbox(new_value) => {
                if let Some(reservation) = self.new_reservation.as_mut() {
                    reservation.occupied = new_value;
                }
            }
            // Asks to add a room to the vec of booked rooms of the current add reservation
            Message::AddReservationRoom(room_id, room_price) => {
                if let Some(reservation) = self.new_reservation.as_mut() {
                    let room_already_exists = reservation
                        .rooms
                        .iter()
                        .any(|sold_room| sold_room.room_id == Some(room_id));

                    if !room_already_exists {
                        reservation.rooms.push(SoldRoom {
                            id: None,
                            room_id: Some(room_id),
                            guests: Vec::new(),
                            price: room_price,
                            invoices: Vec::new(),
                        });
                    }
                }
            }
            // Asks to remove a room to the vec of booked rooms of the current add reservation
            Message::RemoveReservationRoom(room_id) => {
                if let Some(reservation) = self.new_reservation.as_mut() {
                    reservation
                        .rooms
                        .retain(|room| room.room_id != Some(room_id) || !room.invoices.is_empty());
                }
            }
            // Tries to add the current reservation to the database
            Message::AddReservation => {
                if let Some(reservation) = &self.new_reservation {
                    let valid = is_new_reservation_valid(reservation);
                    if valid {
                        action.add_instruction(AddReservationsInstruction::TryAddReservation(
                            reservation.clone(),
                        ));
                    }
                }
            }

            // Asks to open the client selector page/component
            Message::OpenClientSelector => {
                if self.database.is_some() {
                    self.clients_selector.page_mode = ClientsPageMode::Selection;
                    self.clients_selector.database = self.database.clone();
                    let clients_action = self.update(Message::Clients(clients::Message::InitPage));
                    action.tasks.extend(clients_action.tasks);
                    self.page_mode = AddReservationPageMode::PickingClient;
                }
            }
            // Messages of the clients (selector) page
            Message::Clients(message) => {
                let client_action = self.clients_selector.update(message);

                let clients_tasks: Vec<Task<Message>> = client_action
                    .tasks
                    .into_iter()
                    .map(|task| task.map(Message::Clients))
                    .collect();
                action.tasks.extend(clients_tasks);

                for instructions in client_action.instructions {
                    match instructions {
                        clients::ClientsInstruction::Back => {
                            let _ = self.update(Message::Back);
                        }
                        clients::ClientsInstruction::ClientSelected(client) => {
                            if let Some(reservation) = self.new_reservation.as_mut() {
                                reservation.client_id = client.id;
                                reservation.client_name = format!(
                                    "{} {} {} | {}",
                                    client.name,
                                    client.first_surname,
                                    client.second_surname,
                                    client.country
                                );
                                let _ = self.update(Message::Back);
                            }
                        }
                    }
                }
            }
        }

        action
    }

    const GLOBAL_SPACING: f32 = 6.;
    const GLOBAL_BUTTON_HEIGHT: f32 = 60.;
    const TITLE_TEXT_SIZE: f32 = 25.0;
    const TEXT_SIZE: f32 = 18.0;

    /// Returns the view of the subscreen
    pub fn view(&self) -> Element<Message> {
        match self.page_mode {
            AddReservationPageMode::AddingReservation => self.view_add_reservation_form(),
            AddReservationPageMode::PickingClient => {
                self.clients_selector.view().map(Message::Clients)
            }
        }
    }

    //
    //  VIEW COMPOSING
    //

    /// Returns the view of the header row of the subscreen
    pub fn view_header_row(&self) -> Element<Message> {
        if self.page_mode == AddReservationPageMode::AddingReservation {
            let spacing = Pixels::from(Self::GLOBAL_SPACING);
            let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

            let back_button = widget::Button::new(
                widget::Text::new(fl!("back"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::Back)
            .height(button_height);

            widget::Row::new()
                .push(back_button)
                .push(
                    widget::Text::new(fl!("add-reservation"))
                        .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                        .align_y(Alignment::Center),
                )
                .push(widget::Space::new(Length::Fill, Length::Shrink))
                .width(Length::Fill)
                .align_y(Alignment::Center)
                .spacing(spacing)
                .into()
        } else {
            widget::Space::new(Length::Shrink, Length::Shrink).into() // TODO: This moves the client selector page a little bit down, improve this
        }
    }

    /// Returns the view of the header row of the subscreen
    fn view_add_reservation_form(&self) -> Element<Message> {
        if let Some(new_reservation) = &self.new_reservation {
            if new_reservation.id.is_none() {
                let spacing = Pixels::from(Self::GLOBAL_SPACING);

                // Entry Date
                let entry_date_label =
                    widget::Text::new(format!("{} (yyyy-mm-dd)", fl!("entry-date")))
                        .width(Length::Fill);
                let entry_date_iced_aw_date = Date {
                    year: new_reservation.entry_date.unwrap_or_default().year(),
                    month: new_reservation.entry_date.unwrap_or_default().month(),
                    day: new_reservation.entry_date.unwrap_or_default().day(),
                };
                let entry_date_picker = DatePicker::new(
                    self.new_reservation_datepickers_state
                        .show_entry_date_picker,
                    entry_date_iced_aw_date,
                    widget::Button::new(widget::Text::new(fl!("edit"))).on_press(
                        Message::ShowDatePicker(ReservationDateInputFields::EntryDate),
                    ),
                    Message::CancelDateOperation,
                    |date| Message::UpdateDateField(date, ReservationDateInputFields::EntryDate),
                );
                let entry_date_input = widget::TextInput::new(
                    fl!("entry-date").as_str(),
                    &entry_date_iced_aw_date.to_string(),
                )
                .style(|t, _| widget::text_input::default(t, widget::text_input::Status::Active))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);
                let entry_date_input_row = widget::Row::new()
                    .push(entry_date_input)
                    .push(entry_date_picker)
                    .align_y(Alignment::Center)
                    .spacing(1.);

                let entry_date_input_column = widget::Column::new()
                    .push(entry_date_label)
                    .push(entry_date_input_row)
                    .width(Length::Fill)
                    .spacing(1.);

                // Departure Date
                let departure_date_label =
                    widget::Text::new(format!("{} (yyyy-mm-dd)", fl!("departure-date")))
                        .width(Length::Fill);
                let departure_date_iced_aw_date = Date {
                    year: new_reservation.departure_date.unwrap_or_default().year(),
                    month: new_reservation.departure_date.unwrap_or_default().month(),
                    day: new_reservation.departure_date.unwrap_or_default().day(),
                };
                let departure_date_picker = DatePicker::new(
                    self.new_reservation_datepickers_state
                        .show_departure_date_picker,
                    departure_date_iced_aw_date,
                    widget::Button::new(widget::Text::new(fl!("edit"))).on_press(
                        Message::ShowDatePicker(ReservationDateInputFields::DepartureDate),
                    ),
                    Message::CancelDateOperation,
                    |date| {
                        Message::UpdateDateField(date, ReservationDateInputFields::DepartureDate)
                    },
                );
                let departure_date_input = widget::TextInput::new(
                    fl!("departure-date").as_str(),
                    &departure_date_iced_aw_date.to_string(),
                )
                .style(|t, _| widget::text_input::default(t, widget::text_input::Status::Active))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);
                let departure_date_input_row = widget::Row::new()
                    .push(departure_date_input)
                    .push(departure_date_picker)
                    .align_y(Alignment::Center)
                    .spacing(1.);

                let departure_date_input_column = widget::Column::new()
                    .push(departure_date_label)
                    .push(departure_date_input_row)
                    .width(Length::Fill)
                    .spacing(1.);

                // Occupied
                let occupied = widget::Checkbox::new(fl!("occupied"), new_reservation.occupied)
                    .text_size(Pixels::from(Self::TEXT_SIZE))
                    .on_toggle(Message::ToggleOccupiedCheckbox);

                // Rooms Selector
                let available_rooms = self
                    .rooms
                    .iter()
                    .filter(|room| {
                        !self.reservations.iter().any(|reservation| {
                            reservation.rooms.iter().any(|r| r.room_id == room.id)
                                && reservation.entry_date.unwrap()
                                    < new_reservation.departure_date.unwrap()
                                && reservation.departure_date.unwrap()
                                    > new_reservation.entry_date.unwrap()
                        })
                    })
                    .cloned()
                    .collect::<Vec<Room>>();
                let rooms_label = widget::Text::new(fl!("rooms")).width(Length::Fill);
                let selected_room = available_rooms.first().cloned();
                let rooms_selector = widget::PickList::new(available_rooms, selected_room, |r| {
                    Message::AddReservationRoom(r.id.unwrap(), r.default_room_price)
                })
                .width(Length::Fill);
                let rooms_selector_column = widget::Column::new()
                    .push(rooms_label)
                    .push(rooms_selector)
                    .width(Length::Fill)
                    .spacing(1.);

                // Already Selected Rooms
                let mut reservation_rooms_column = widget::Column::new()
                    .push(widget::Text::new(fl!("rooms")))
                    .width(Length::Fill)
                    .spacing(spacing);
                for sold_room in &new_reservation.rooms {
                    let room = self.rooms.iter().find(|r| r.id == sold_room.room_id);
                    if let Some(room) = room {
                        reservation_rooms_column = reservation_rooms_column.push(
                            widget::Row::new()
                                .push(widget::Text::new(&room.name).width(Length::Fill))
                                .push(widget::Button::new("X").on_press(
                                    Message::RemoveReservationRoom(room.id.unwrap_or_default()),
                                ))
                                .align_y(Alignment::Center)
                                .width(Length::Fill),
                        )
                    }
                }

                // Client Selection
                let client_text = if new_reservation.client_name.is_empty() {
                    fl!("no-client-selected")
                } else {
                    new_reservation.client_name.clone()
                };
                let client_selection_row = widget::Row::new()
                    .push(
                        widget::Text::new(client_text)
                            .size(Self::TEXT_SIZE)
                            .width(Length::Fill),
                    )
                    .push(
                        widget::Button::new(
                            widget::Text::new(fl!("select"))
                                .width(Length::Shrink)
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .width(Length::Shrink)
                        .on_press(Message::OpenClientSelector),
                    )
                    .align_y(Alignment::Center)
                    .width(Length::Fill);
                let client_selection_col = widget::Column::new()
                    .push(widget::Text::new(fl!("main-client")).width(Length::Fill))
                    .push(client_selection_row)
                    .width(Length::Fill)
                    .spacing(1.);

                // Submit
                let submit_button = widget::Button::new(
                    widget::Text::new(fl!("add"))
                        .width(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::AddReservation)
                .width(Length::Fill);

                // Layout
                let date_inputs = widget::Column::new()
                    .push(entry_date_input_column)
                    .push(departure_date_input_column)
                    .push(occupied)
                    .spacing(spacing)
                    .width(Length::Fill);
                let rooms_col = widget::Column::new()
                    .push(rooms_selector_column)
                    .push(reservation_rooms_column)
                    .spacing(spacing)
                    .width(Length::Fill);
                let second_row = widget::Row::new()
                    .push(date_inputs)
                    .push(rooms_col)
                    .spacing(spacing)
                    .align_y(Alignment::Start)
                    .width(Length::Fill);

                let result = widget::Column::new()
                    .push(client_selection_col)
                    .push(second_row)
                    .push(submit_button)
                    .spacing(spacing)
                    .width(Length::Fixed(850.));

                widget::Container::new(result)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .padding(Padding::new(50.))
                    .into()
            } else {
                widget::Container::new(widget::Text::new(
                    "Error, NewReservation improperly initialized...",
                ))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .padding(Padding::new(50.))
                .into()
            }
        } else {
            widget::Container::new(widget::Text::new(
                "Error, NewReservation not initialized...",
            ))
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

fn is_new_reservation_valid(reservation: &Reservation) -> bool {
    if reservation.client_id.is_none() {
        return false;
    }
    if reservation.rooms.is_empty() {
        return false;
    }
    if reservation.entry_date.is_none() {
        return false;
    }
    if reservation.departure_date.is_none() {
        return false;
    }

    true
}
