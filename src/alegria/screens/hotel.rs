// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{Alignment, Element, Length, Pixels, Task, widget};
use sqlx::{Pool, Sqlite};

use crate::{alegria::action::AlegriaAction, fl};

use super::hotel_subscreens::{
    reservations::{self, Reservations},
    room_types::{self, RoomTypes},
};

#[derive(Debug, Clone)]
pub enum SubScreen {
    Home,
    Reservations,
    RoomTypes,
}

pub struct Hotel {
    /// Database of the application
    database: Option<Arc<Pool<Sqlite>>>,
    /// Represents a SubScreen of the Reservations Page
    sub_screen: SubScreen,
    /// Reservations Subscreen of the HotelPage
    reservations: Reservations,
    /// RoomTypes Subscreen of the HotelPage
    room_types: RoomTypes,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back
    ChangeSubScreen(SubScreen),

    Reservations(reservations::Message),
    RoomTypes(room_types::Message),
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum HotelInstruction {
    Back, // Asks the parent (app.rs) to go back
}

impl Hotel {
    /// Initializes the bar screen
    pub fn init() -> Self {
        Self {
            database: None,
            sub_screen: SubScreen::Home,
            reservations: Reservations::init(),
            room_types: RoomTypes::init(),
        }
    }

    pub fn set_database(&mut self, database: Option<Arc<Pool<Sqlite>>>) {
        self.database = database.clone();
        self.room_types.database = database.clone();
        self.reservations.database = database;
    }

    /// Cleans the state of the bar screen preserving the database
    /// intended to be called when switching to another screen in order to save memory.
    pub fn clean_state(database: Option<Arc<Pool<Sqlite>>>) -> Self {
        Self {
            database: database.clone(),
            sub_screen: SubScreen::Home,
            reservations: Reservations::clean_state(database.clone()),
            room_types: RoomTypes::clean_state(database),
        }
    }

    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<HotelInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            Message::Back => action.add_instruction(HotelInstruction::Back),

            Message::ChangeSubScreen(sub_screen) => match sub_screen {
                SubScreen::Home => {
                    self.sub_screen = sub_screen;
                    self.reservations =
                        reservations::Reservations::clean_state(self.database.clone());
                    self.room_types = room_types::RoomTypes::clean_state(self.database.clone());
                }
                SubScreen::Reservations => {
                    self.sub_screen = sub_screen;
                }
                SubScreen::RoomTypes => {
                    self.sub_screen = sub_screen;
                    let room_types_action =
                        self.update(Message::RoomTypes(room_types::Message::FetchRoomTypes));
                    action.tasks.extend(room_types_action.tasks);
                }
            },

            Message::Reservations(message) => {
                let reservation_action = self.reservations.update(message);

                let reservations_tasks: Vec<Task<Message>> = reservation_action
                    .tasks
                    .into_iter()
                    .map(|task| task.map(Message::Reservations))
                    .collect();
                action.tasks.extend(reservations_tasks);

                for reservations_instructions in reservation_action.instructions {
                    match reservations_instructions {
                        reservations::ReservationsInstruction::Back => {
                            let _ = self.update(Message::ChangeSubScreen(SubScreen::Home));
                        }
                    }
                }
            }

            Message::RoomTypes(message) => {
                let room_type_action = self.room_types.update(message);

                let room_types_tasks: Vec<Task<Message>> = room_type_action
                    .tasks
                    .into_iter()
                    .map(|task| task.map(Message::RoomTypes))
                    .collect();
                action.tasks.extend(room_types_tasks);

                for instructions in room_type_action.instructions {
                    match instructions {
                        room_types::RoomTypesInstruction::Back => {
                            let _ = self.update(Message::ChangeSubScreen(SubScreen::Home));
                        }
                    }
                }
            }
        };

        action
    }

    const GLOBAL_SPACING: f32 = 6.;
    const GLOBAL_BUTTON_HEIGHT: f32 = 60.;
    const SQUAREBUTTONXY: f32 = 120.;

    /// Returns the view of the hotel screen
    pub fn view(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        let content = match self.sub_screen {
            SubScreen::Home => {
                // HEADER
                let header_row = self.view_header_row();

                let buttons_row = widget::Row::new()
                    .push(
                        widget::Button::new(
                            widget::Text::new(fl!("reservations"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::ChangeSubScreen(SubScreen::Reservations))
                        .width(Length::Fixed(Self::SQUAREBUTTONXY))
                        .height(Length::Fixed(Self::SQUAREBUTTONXY)),
                    )
                    .push(
                        widget::Button::new(
                            widget::Text::new(fl!("room-types"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::ChangeSubScreen(SubScreen::RoomTypes))
                        .width(Length::Fixed(Self::SQUAREBUTTONXY))
                        .height(Length::Fixed(Self::SQUAREBUTTONXY)),
                    )
                    .spacing(Pixels::from(5.));

                let content = widget::Container::new(buttons_row)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill);

                widget::Column::new()
                    .push(header_row)
                    .push(content)
                    .spacing(spacing)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .into()
            }
            SubScreen::Reservations => self.reservations.view().map(Message::Reservations),
            SubScreen::RoomTypes => self.room_types.view().map(Message::RoomTypes),
        };

        widget::Container::new(content)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    //
    //  VIEW COMPOSING
    //

    const TITLE_TEXT_SIZE: f32 = 25.0;

    /// Returns the view of the header row of the hotel screen
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

        widget::Row::new()
            .push(back_button)
            .push(
                widget::Text::new(fl!("hotel"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .spacing(spacing)
            .into()
    }

    //
    //  END OF VIEW COMPOSING
    //
}
