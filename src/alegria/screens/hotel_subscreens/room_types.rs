// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{Alignment, Element, Length, Pixels, Task, widget};
use sqlx::{Pool, Sqlite};

use crate::{
    alegria::{action::AlegriaAction, core::models::room_type::RoomType},
    fl,
};

pub struct RoomTypes {
    /// Database of the application
    pub database: Option<Arc<Pool<Sqlite>>>,
    /// Holds the state of all the room_types
    pub room_types: Vec<RoomType>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back

    FetchRoomTypes,              // Fetches all the current roomtypes
    SetRoomTypes(Vec<RoomType>), // Sets the roomtypes on the app state
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
            room_types: Vec::new(),
        }
    }

    /// Cleans the state of the screen preserving the database
    /// intended to be called when switching to another screen in order to save memory.
    pub fn clean_state(database: Option<Arc<Pool<Sqlite>>>) -> Self {
        Self {
            database,
            room_types: Vec::new(),
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

        // ROOM TYPES GRID
        let room_types_grid = self.view_room_types_grid();

        widget::Column::new()
            .push(header_row)
            .push(room_types_grid)
            .spacing(spacing)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    //
    //  VIEW COMPOSING
    //

    /// Returns the view of the header row of the subscreen
    fn view_header_row(&self) -> Element<Message> {
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
            .width(Length::Fill)
            .into()
    }

    /// Returns the view of the room types grid
    fn view_room_types_grid(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        let mut grid = widget::Column::new().spacing(spacing).width(Length::Fill);
        let mut current_row = widget::Row::new().spacing(spacing).width(Length::Fill);
        for room_type in &self.room_types {
            current_row = current_row.push(widget::Text::new(&room_type.name));

            grid = grid.push(current_row);
            current_row = widget::Row::new().spacing(spacing).width(Length::Fill);
        }

        widget::Container::new(grid).width(Length::Fill).into()
    }

    //
    //  END OF VIEW COMPOSING
    //
}
