// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{
    Alignment, Element, Length, Pixels, Task,
    widget::{
        Column, PickList, Row, Rule, Space, button, column, container, row, text, text_input,
    },
};
use sqlx::PgPool;

use crate::{
    alegria::{
        action::AlegriaAction,
        core::models::{room::Room, room_type::RoomType},
    },
    fl,
};

#[derive(Debug, Clone, PartialEq)]
enum RoomsScreen {
    List,
    AddEdit,
}

#[derive(Debug, Clone)]
pub enum RoomTextInputFields {
    Name,
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

pub struct Rooms {
    /// Database of the application
    pub database: Option<Arc<PgPool>>,
    /// Determines which is the current view of the subscreen
    current_screen: RoomsScreen,
    /// Holds the state of all the rooms
    rooms: Vec<Room>,
    /// Holds the state of all the room types, needed for the room type selected
    room_types: Vec<RoomType>,
    /// Holds the state of the current editing/adding Room
    add_edit_room: Option<Room>,
    /// Holds the pagination state and config for this page
    pagination_state: PaginationConfig,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back

    FetchRooms,          // Fetches all the current rooms
    SetRooms(Vec<Room>), // Sets the rooms on the app state

    FetchRoomTypes,              // Fetches all the current room types
    SetRoomTypes(Vec<RoomType>), // Sets the room types on the app state

    AskEditRoom(Room), // Callback after asking to edit a room, changes the screen and the add_edit_room state
    AskAddRoom, // Callback after asking to add a room, changes the screen and the add_edit_room state
    CancelRoomOperation, // Callback after asking to cancel an add or an edit

    TextInputUpdate(String, RoomTextInputFields), // Callback when using the text inputs to add or edit a room
    UpdatedSelectedRoomTypeId(i32), // Callback after selecting a new RoomTypeId for the current room

    AddCurrentRoom,      // Tries to Add the current room to the database
    EditCurrentRoom,     // Tries to Edit the current room to the database
    DeleteCurrentRoom,   // Tries to delete the current Room
    ModifiedCurrentRoom, // Callback after delete/update/add of a current Room

    PaginationAction(PaginationAction), // Try to go left or right a page on the grid
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum RoomsInstruction {
    Back, // Asks the parent (app.rs) to go back
}

impl Default for Rooms {
    fn default() -> Self {
        Self {
            database: None,
            current_screen: RoomsScreen::List,
            rooms: Vec::new(),
            room_types: Vec::new(),
            add_edit_room: None,
            pagination_state: PaginationConfig::default(),
        }
    }
}

impl Rooms {
    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<RoomsInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            Message::Back => action.add_instruction(RoomsInstruction::Back),

            // Fetches all the current rooms
            Message::FetchRooms => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        Room::get_all(pool.clone()),
                        |res| match res {
                            Ok(res) => Message::SetRooms(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetRooms(Vec::new())
                            }
                        },
                    ));
                }
            }
            // Sets the rooms on the app state
            Message::SetRooms(res) => {
                self.rooms = res;
            }

            // Fetches all the current room types
            Message::FetchRoomTypes => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        RoomType::get_all(pool.clone()),
                        |res| match res {
                            Ok(res) => Message::SetRoomTypes(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetRoomTypes(Vec::new())
                            }
                        },
                    ));
                }
            }
            // Sets the room types on the app state
            Message::SetRoomTypes(res) => {
                self.room_types = res;
            }

            // Callback after asking to edit a room, changes the screen and the add_edit_room state
            Message::AskEditRoom(room) => {
                self.add_edit_room = Some(room);
                self.current_screen = RoomsScreen::AddEdit;
            }
            // Callback after asking to edit a room, changes the screen and the add_edit_room state
            Message::AskAddRoom => {
                self.add_edit_room = Some(Room::default());
                self.current_screen = RoomsScreen::AddEdit;
            }
            // Callback after asking to cancel an add or an edit
            Message::CancelRoomOperation => {
                self.add_edit_room = None;
                self.current_screen = RoomsScreen::List;
                return self.update(Message::FetchRooms);
            }

            // Callback when using the text inputs to add or edit a room
            Message::TextInputUpdate(new_value, field) => {
                if let Some(room) = self.add_edit_room.as_mut() {
                    match field {
                        RoomTextInputFields::Name => {
                            room.name = new_value;
                        }
                    }
                }
            }
            // Callback after selecting a new RoomTypeId for the current room
            Message::UpdatedSelectedRoomTypeId(new_id) => {
                if let Some(room) = &mut self.add_edit_room {
                    room.room_type_id = Some(new_id);
                }
            }

            // Tries to Add the current room to the database
            Message::AddCurrentRoom => {
                if let Some(room) = &self.add_edit_room {
                    // TODO: Proper validation
                    if !room.name.is_empty() && room.room_type_id.is_some() && room.id.is_none() {
                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                Room::add(pool.clone(), room.clone()),
                                |res| match res {
                                    Ok(_) => Message::ModifiedCurrentRoom,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::CancelRoomOperation
                                    }
                                },
                            ));
                        }
                    }
                }
            }
            // Tries to Edit the current room to the database
            Message::EditCurrentRoom => {
                if let Some(room) = &self.add_edit_room {
                    // TODO: Proper validation
                    if !room.name.is_empty() && room.room_type_id.is_some() && room.id.is_some() {
                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                Room::edit(pool.clone(), room.clone()),
                                |res| match res {
                                    Ok(_) => Message::ModifiedCurrentRoom,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::CancelRoomOperation
                                    }
                                },
                            ));
                        }
                    }
                }
            }
            // Tries to delete the current Room
            Message::DeleteCurrentRoom => {
                if let Some(room) = &self.add_edit_room {
                    if room.id.is_some() {
                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                Room::delete(pool.clone(), room.id.unwrap_or_default()),
                                |res| match res {
                                    Ok(_) => Message::ModifiedCurrentRoom,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::CancelRoomOperation
                                    }
                                },
                            ));
                        }
                    }
                }
            }
            // Callback after add/update/delete of the current Room
            Message::ModifiedCurrentRoom => {
                self.add_edit_room = None;
                self.current_screen = RoomsScreen::List;
                return self.update(Message::FetchRooms);
            }

            // Try to go left or right a page on the grid
            Message::PaginationAction(action) => match action {
                PaginationAction::Back => {
                    if self.pagination_state.current_page > 0 {
                        self.pagination_state.current_page -= 1;
                    }
                }
                PaginationAction::Forward => {
                    let next_page_start = (self.pagination_state.current_page + 1)
                        * self.pagination_state.items_per_page;
                    if next_page_start < self.rooms.len().try_into().unwrap_or_default() {
                        self.pagination_state.current_page += 1;
                    }
                }
            },
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
            RoomsScreen::List => self.view_rooms_grid(),
            RoomsScreen::AddEdit => self.view_add_edit_screen(),
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
            RoomsScreen::List => button(
                text(fl!("add"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::AskAddRoom)
            .height(button_height),
            RoomsScreen::AddEdit => button(
                text(fl!("cancel"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .style(button::danger)
            .on_press(Message::CancelRoomOperation)
            .height(button_height),
        };

        let delete_button = button(
            text(fl!("delete"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .style(button::secondary)
        .on_press(Message::DeleteCurrentRoom)
        .height(button_height);

        let mut result_row = Row::new();
        if self.current_screen == RoomsScreen::List {
            result_row = result_row.push(back_button);
        }

        result_row = result_row
            .push(
                text(fl!("rooms"))
                    .size(Self::TITLE_TEXT_SIZE)
                    .align_y(Alignment::Center),
            )
            .push(Space::new(Length::Fill, Length::Shrink));

        if self.current_screen == RoomsScreen::AddEdit
            && self.add_edit_room.as_ref().is_some_and(|x| x.id.is_some())
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

    /// Returns the view of the room types grid
    fn view_rooms_grid(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        if self.rooms.is_empty() {
            return container(text(fl!("no-rooms")).size(Self::TITLE_TEXT_SIZE))
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .padding(50.)
                .into();
        }

        let title_row = Row::new()
            .push(
                text(fl!("name"))
                    .size(Self::TITLE_TEXT_SIZE)
                    .width(300.)
                    .align_y(Alignment::Center),
            )
            .push(
                text(fl!("room-type"))
                    .size(Self::TITLE_TEXT_SIZE)
                    .width(200.)
                    .align_y(Alignment::Center),
            )
            .push(
                text(fl!("edit"))
                    .size(Self::TITLE_TEXT_SIZE)
                    .width(200.)
                    .align_y(Alignment::Center)
                    .align_x(Alignment::End),
            )
            .width(Length::Shrink)
            .align_y(Alignment::Center);

        // Calculate the indices for the current page
        let start_index: usize = self.pagination_state.current_page as usize
            * self.pagination_state.items_per_page as usize;
        let end_index = usize::min(
            start_index + self.pagination_state.items_per_page as usize,
            self.rooms.len(),
        );

        let mut grid = Column::new()
            .push(title_row)
            .align_x(Alignment::Center)
            .spacing(spacing)
            .width(Length::Shrink);

        for room in &self.rooms[start_index..end_index] {
            let row = Row::new()
                .width(Length::Shrink)
                .push(
                    text(&room.name)
                        .size(Self::TEXT_SIZE)
                        .width(300.)
                        .align_y(Alignment::Center),
                )
                .push(
                    text(&room.room_type_name)
                        .size(Self::TEXT_SIZE)
                        .width(200.)
                        .align_y(Alignment::Center),
                )
                .push(
                    container(
                        button(
                            text(fl!("edit"))
                                .size(Self::TEXT_SIZE)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::AskEditRoom(room.clone()))
                        .width(Length::Shrink),
                    )
                    .width(200.)
                    .align_x(Alignment::End)
                    .align_y(Alignment::Center),
                )
                .align_y(Alignment::Center);

            // Limit Rule size to sum of all column widths
            grid = grid.push(row![Rule::horizontal(1.)].width(Length::Fixed(700.)));
            grid = grid.push(row);
        }

        grid = grid.push(row![Rule::horizontal(1.)].width(700.));

        grid = grid.push(text(format!(
            "{} {}",
            fl!("page").as_str(),
            &self.pagination_state.current_page + 1
        )));
        grid = grid.push(Space::with_height(Length::Fill));
        grid = grid.push(
            Row::new()
                .width(Length::Fixed(850.))
                .push(
                    button(
                        text(fl!("back"))
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                            .width(Length::Fill)
                            .height(button_height),
                    )
                    .on_press(Message::PaginationAction(PaginationAction::Back)),
                )
                .push(
                    button(
                        text(fl!("next"))
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                            .width(Length::Fill)
                            .height(button_height),
                    )
                    .on_press(Message::PaginationAction(PaginationAction::Forward)),
                )
                .spacing(spacing),
        );

        container(grid)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding(50.)
            .into()
    }

    /// Returns the view of the room types add/edit screen
    fn view_add_edit_screen(&self) -> Element<Message> {
        if let Some(room) = &self.add_edit_room {
            let spacing = Pixels::from(Self::GLOBAL_SPACING);

            let name_label = text(fl!("name")).width(Length::Fill);
            let name_input = text_input(fl!("name").as_str(), &room.name)
                .on_input(|c| Message::TextInputUpdate(c, RoomTextInputFields::Name))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);

            let room_type_label = text(fl!("room-type")).width(Length::Fill);
            let selected = self.room_types.iter().find(|rt| rt.id == room.room_type_id);
            let room_type_selector =
                PickList::new(self.room_types.clone(), selected, |room_type| {
                    Message::UpdatedSelectedRoomTypeId(room_type.id.unwrap_or_default())
                })
                .width(Length::Fill);

            let submit_button = if room.id.is_some() {
                button(
                    text(fl!("edit"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(Self::TEXT_SIZE),
                )
                .on_press(Message::EditCurrentRoom)
                .width(Length::Fill)
            } else {
                button(
                    text(fl!("add"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(Self::TEXT_SIZE),
                )
                .on_press(Message::AddCurrentRoom)
                .width(Length::Fill)
            };

            let name_input_column = column![name_label, name_input].width(700.).spacing(1.);

            let room_type_input_column = column![room_type_label, room_type_selector]
                .width(700.)
                .spacing(1.);

            let form_column = column![name_input_column, room_type_input_column, submit_button]
                .width(700.)
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
