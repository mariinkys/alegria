// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{
    Alignment, Element, Length, Padding, Pixels, Task,
    widget::{self, Space},
};
use sqlx::PgPool;

use crate::{
    alegria::{action::AlegriaAction, core::models::room_type::RoomType},
    fl,
};

#[derive(Debug, Clone, PartialEq)]
enum RoomTypesScreen {
    List,
    AddEdit,
}

#[derive(Debug, Clone)]
pub enum RoomTypeTextInputFields {
    Name,
    Price,
}

pub struct RoomTypes {
    /// Database of the application
    pub database: Option<Arc<PgPool>>,
    /// Determines which is the current view of the subscreen
    current_screen: RoomTypesScreen,
    /// Holds the state of all the room_types
    room_types: Vec<RoomType>,
    /// Holds the state of the current editing/adding RoomType
    add_edit_room_type: Option<RoomType>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back

    FetchRoomTypes,              // Fetches all the current roomtypes
    SetRoomTypes(Vec<RoomType>), // Sets the roomtypes on the app state

    AskEditRoomType(RoomType), // Callback after asking to edit a room type, changes the screen and the add_edit_room type state
    AskAddRoomType, // Callback after asking to edit a room type, changes the screen and the add_edit_room type state
    CancelRoomTypeOperation, // Callback after asking to cancel an add or an edit

    TextInputUpdate(String, RoomTypeTextInputFields), // Callback when using the text inputs to add or edit a room type

    AddCurrentRoomType,      // Tries to Add the current room type to the database
    EditCurrentRoomType,     // Tries to Edit the current room type to the database
    DeleteCurrentRoomType,   // Tries to delete the current RoomType
    ModifiedCurrentRoomType, // Callback after delete/update/add of a current RoomType
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum RoomTypesInstruction {
    Back, // Asks the parent (app.rs) to go back
}

impl RoomTypes {
    /// Initializes the screen
    pub fn init() -> Self {
        Self {
            database: None,
            current_screen: RoomTypesScreen::List,
            room_types: Vec::new(),
            add_edit_room_type: None,
        }
    }

    /// Cleans the state of the screen preserving the database
    /// intended to be called when switching to another screen in order to save memory.
    pub fn clean_state(database: Option<Arc<PgPool>>) -> Self {
        Self {
            database,
            current_screen: RoomTypesScreen::List,
            room_types: Vec::new(),
            add_edit_room_type: None,
        }
    }

    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<RoomTypesInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            Message::Back => action.add_instruction(RoomTypesInstruction::Back),

            // Fetches all the current roomtypes
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
            // Sets the roomtypes on the app state
            Message::SetRoomTypes(res) => {
                self.room_types = res;
            }

            // Callback after asking to edit a room type, changes the screen and the add_edit_room type state
            Message::AskEditRoomType(room_type) => {
                self.add_edit_room_type = Some(room_type);
                self.current_screen = RoomTypesScreen::AddEdit;
            }
            // Callback after asking to edit a room type, changes the screen and the add_edit_room type state
            Message::AskAddRoomType => {
                self.add_edit_room_type = Some(RoomType::default());
                self.current_screen = RoomTypesScreen::AddEdit;
            }
            // Callback after asking to cancel an add or an edit
            Message::CancelRoomTypeOperation => {
                self.add_edit_room_type = None;
                self.current_screen = RoomTypesScreen::List;
                return self.update(Message::FetchRoomTypes);
            }

            // Callback when using the text inputs to add or edit a room type
            Message::TextInputUpdate(new_value, field) => {
                if let Some(room_type) = self.add_edit_room_type.as_mut() {
                    match field {
                        RoomTypeTextInputFields::Name => {
                            room_type.name = new_value;
                        }
                        RoomTypeTextInputFields::Price => {
                            // We ignore the input if we already have two decimals and we're trying to add more
                            let ignore_action = new_value.len() > room_type.price_input.len()
                                && room_type
                                    .price_input
                                    .find('.')
                                    .is_some_and(|idx| room_type.price_input.len() - idx > 2);

                            if !ignore_action {
                                if let Ok(num) = new_value.parse::<f32>() {
                                    room_type.price = Some(num);
                                    room_type.price_input = new_value;
                                } else if new_value.is_empty() {
                                    room_type.price = Some(0.0);
                                    room_type.price_input = new_value;
                                }
                            }
                        }
                    }
                }
            }

            // Tries to Add the current room type to the database
            Message::AddCurrentRoomType => {
                if let Some(room_type) = &self.add_edit_room_type {
                    // TODO: Proper validation
                    if !room_type.name.is_empty()
                        && room_type.price.is_some()
                        && room_type.id.is_none()
                    {
                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                RoomType::add(pool.clone(), room_type.clone()),
                                |res| match res {
                                    Ok(_) => Message::ModifiedCurrentRoomType,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::CancelRoomTypeOperation
                                    }
                                },
                            ));
                        }
                    }
                }
            }
            // Tries to Edit the current room type to the database
            Message::EditCurrentRoomType => {
                if let Some(room_type) = &self.add_edit_room_type {
                    // TODO: Proper validation
                    if !room_type.name.is_empty()
                        && room_type.price.is_some()
                        && room_type.id.is_some()
                    {
                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                RoomType::edit(pool.clone(), room_type.clone()),
                                |res| match res {
                                    Ok(_) => Message::ModifiedCurrentRoomType,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::CancelRoomTypeOperation
                                    }
                                },
                            ));
                        }
                    }
                }
            }
            // Tries to delete the current RoomType
            Message::DeleteCurrentRoomType => {
                if let Some(room_type) = &self.add_edit_room_type {
                    if room_type.id.is_some() {
                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                RoomType::delete(pool.clone(), room_type.id.unwrap_or_default()),
                                |res| match res {
                                    Ok(_) => Message::ModifiedCurrentRoomType,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::CancelRoomTypeOperation
                                    }
                                },
                            ));
                        }
                    }
                }
            }
            // Callback after add/update/delete of the current RoomType
            Message::ModifiedCurrentRoomType => {
                self.add_edit_room_type = None;
                self.current_screen = RoomTypesScreen::List;
                return self.update(Message::FetchRoomTypes);
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
            RoomTypesScreen::List => self.view_room_types_grid(),
            RoomTypesScreen::AddEdit => self.view_add_edit_screen(),
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
            RoomTypesScreen::List => widget::Button::new(
                widget::Text::new(fl!("add"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::AskAddRoomType)
            .height(button_height),
            RoomTypesScreen::AddEdit => widget::Button::new(
                widget::Text::new(fl!("cancel"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .style(widget::button::danger)
            .on_press(Message::CancelRoomTypeOperation)
            .height(button_height),
        };

        let delete_button = widget::Button::new(
            widget::Text::new(fl!("delete"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .style(widget::button::secondary)
        .on_press(Message::DeleteCurrentRoomType)
        .height(button_height);

        let mut result_row = widget::Row::new();
        if self.current_screen == RoomTypesScreen::List {
            result_row = result_row.push(back_button);
        }

        result_row = result_row
            .push(
                widget::Text::new(fl!("room-types"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .align_y(Alignment::Center),
            )
            .push(Space::new(Length::Fill, Length::Shrink));

        if self.current_screen == RoomTypesScreen::AddEdit
            && self
                .add_edit_room_type
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

    /// Returns the view of the room types grid
    fn view_room_types_grid(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        if self.room_types.is_empty() {
            return widget::Container::new(
                widget::Text::new(fl!("no-room-types")).size(Pixels::from(Self::TITLE_TEXT_SIZE)),
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
                widget::Text::new(fl!("price"))
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

        for room_type in &self.room_types {
            let row = widget::Row::new()
                .width(Length::Shrink)
                .push(
                    widget::Text::new(&room_type.name)
                        .size(Pixels::from(Self::TEXT_SIZE))
                        .width(Length::Fixed(300.))
                        .align_y(Alignment::Center),
                )
                .push(
                    widget::Text::new(format!("{:.2} €", room_type.price.unwrap_or(0.)))
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
                        .on_press(Message::AskEditRoomType(room_type.clone()))
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
        if let Some(room_type) = &self.add_edit_room_type {
            let spacing = Pixels::from(Self::GLOBAL_SPACING);

            let name_label = widget::Text::new(fl!("name")).width(Length::Fill);

            let name_input = widget::TextInput::new(fl!("name").as_str(), &room_type.name)
                .on_input(|c| Message::TextInputUpdate(c, RoomTypeTextInputFields::Name))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);

            let price_label = widget::Text::new(fl!("price")).width(Length::Fill);

            let price_input = widget::TextInput::new(fl!("price").as_str(), &room_type.price_input)
                .on_input(|c| Message::TextInputUpdate(c, RoomTypeTextInputFields::Price))
                .size(Pixels::from(Self::TEXT_SIZE))
                .width(Length::Fill);

            let submit_button = if room_type.id.is_some() {
                widget::Button::new(
                    widget::Text::new(fl!("edit"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(Pixels::from(Self::TEXT_SIZE)),
                )
                .on_press(Message::EditCurrentRoomType)
                .width(Length::Fill)
            } else {
                widget::Button::new(
                    widget::Text::new(fl!("add"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .size(Pixels::from(Self::TEXT_SIZE)),
                )
                .on_press(Message::AddCurrentRoomType)
                .width(Length::Fill)
            };

            let name_input_column = widget::Column::new()
                .push(name_label)
                .push(name_input)
                .width(Length::Fixed(700.))
                .spacing(1.);

            let price_input_column = widget::Column::new()
                .push(price_label)
                .push(price_input)
                .width(Length::Fixed(700.))
                .spacing(1.);

            let form_column = widget::Column::new()
                .push(name_input_column)
                .push(price_input_column)
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
