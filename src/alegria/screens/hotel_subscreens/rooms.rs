// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{
    Alignment, Element, Length, Padding, Pixels, Task,
    widget::{self, Space},
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
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum RoomsInstruction {
    Back, // Asks the parent (app.rs) to go back
}

impl Rooms {
    /// Initializes the screen
    pub fn init() -> Self {
        Self {
            database: None,
            current_screen: RoomsScreen::List,
            rooms: Vec::new(),
            room_types: Vec::new(),
            add_edit_room: None,
        }
    }

    /// Cleans the state of the screen preserving the database
    /// intended to be called when switching to another screen in order to save memory.
    pub fn clean_state(database: Option<Arc<PgPool>>) -> Self {
        Self {
            database,
            current_screen: RoomsScreen::List,
            rooms: Vec::new(),
            room_types: Vec::new(),
            add_edit_room: None,
        }
    }

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
            RoomsScreen::List => widget::Button::new(
                widget::Text::new(fl!("add"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::AskAddRoom)
            .height(button_height),
            RoomsScreen::AddEdit => widget::Button::new(
                widget::Text::new(fl!("cancel"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .style(widget::button::danger)
            .on_press(Message::CancelRoomOperation)
            .height(button_height),
        };

        let delete_button = widget::Button::new(
            widget::Text::new(fl!("delete"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .style(widget::button::secondary)
        .on_press(Message::DeleteCurrentRoom)
        .height(button_height);

        let mut result_row = widget::Row::new();
        if self.current_screen == RoomsScreen::List {
            result_row = result_row.push(back_button);
        }

        result_row = result_row
            .push(
                widget::Text::new(fl!("rooms"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
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

        if self.rooms.is_empty() {
            return widget::Container::new(
                widget::Text::new(fl!("no-rooms")).size(Pixels::from(Self::TITLE_TEXT_SIZE)),
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
                    .width(Length::Fixed(300.))
                    .align_y(Alignment::Center),
            )
            .push(
                widget::Text::new(fl!("room-type"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .width(Length::Fixed(200.))
                    .align_y(Alignment::Center),
            )
            .push(
                widget::Text::new(fl!("edit"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .width(Length::Fixed(200.))
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

        for room in &self.rooms {
            let row = widget::Row::new()
                .width(Length::Shrink)
                .push(
                    widget::Text::new(&room.name)
                        .size(Pixels::from(Self::TEXT_SIZE))
                        .width(Length::Fixed(300.))
                        .align_y(Alignment::Center),
                )
                .push(
                    widget::Text::new(&room.room_type_name)
                        .size(Pixels::from(Self::TEXT_SIZE))
                        .width(Length::Fixed(200.))
                        .align_y(Alignment::Center),
                )
                .push(
                    widget::Container::new(
                        widget::Button::new(
                            widget::Text::new(fl!("edit"))
                                .size(Pixels::from(Self::TEXT_SIZE))
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::AskEditRoom(room.clone()))
                        .width(Length::Shrink),
                    )
                    .width(Length::Fixed(200.))
                    .align_x(Alignment::End)
                    .align_y(Alignment::Center),
                )
                .align_y(Alignment::Center);

            // Limit Rule size to sum of all column widths
            grid = grid.push(
                widget::Row::new()
                    .width(Length::Fixed(700.))
                    .push(widget::Rule::horizontal(Pixels::from(1.))),
            );
            grid = grid.push(row);
        }

        grid = grid.push(
            widget::Row::new()
                .width(Length::Fixed(700.))
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
        if let Some(room) = &self.add_edit_room {
            let spacing = Pixels::from(Self::GLOBAL_SPACING);

            let name_label = widget::Text::new(fl!("name")).width(Length::Fill);

            let name_input = widget::TextInput::new(fl!("name").as_str(), &room.name)
                .on_input(|c| Message::TextInputUpdate(c, RoomTextInputFields::Name))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);

            let room_type_label = widget::Text::new(fl!("room-type")).width(Length::Fill);

            let selected = self.room_types.iter().find(|rt| rt.id == room.room_type_id);
            let room_type_selector =
                widget::PickList::new(self.room_types.clone(), selected, |room_type| {
                    Message::UpdatedSelectedRoomTypeId(room_type.id.unwrap_or_default())
                })
                .width(Length::Fill);

            let submit_button = if room.id.is_some() {
                widget::Button::new(
                    widget::Text::new(fl!("edit"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(Pixels::from(Self::TEXT_SIZE)),
                )
                .on_press(Message::EditCurrentRoom)
                .width(Length::Fill)
            } else {
                widget::Button::new(
                    widget::Text::new(fl!("add"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(Pixels::from(Self::TEXT_SIZE)),
                )
                .on_press(Message::AddCurrentRoom)
                .width(Length::Fill)
            };

            let name_input_column = widget::Column::new()
                .push(name_label)
                .push(name_input)
                .width(Length::Fixed(700.))
                .spacing(1.);

            let room_type_input_column = widget::Column::new()
                .push(room_type_label)
                .push(room_type_selector)
                .width(Length::Fixed(700.))
                .spacing(1.);

            let form_column = widget::Column::new()
                .push(name_input_column)
                .push(room_type_input_column)
                .push(submit_button)
                .width(Length::Fixed(700.))
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
