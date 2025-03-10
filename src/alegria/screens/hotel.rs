use std::sync::Arc;

use iced::{Alignment, Element, Length, Pixels, widget};
use sqlx::{Pool, Sqlite};

use crate::{alegria::action::AlegriaAction, fl};

pub struct Hotel {
    /// Database of the application
    pub database: Option<Arc<Pool<Sqlite>>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum HotelInstruction {
    Back, // Asks the parent (app.rs) to go back
}

impl Hotel {
    /// Initializes the bar screen
    pub fn init() -> Self {
        Self { database: None }
    }

    /// Cleans the state of the bar screen preserving the database
    /// intended to be called when switching to another screen in order to save memory.
    pub fn clean_state(database: Option<Arc<Pool<Sqlite>>>) -> Self {
        Self { database }
    }

    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<HotelInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            // Asks the parent (app.rs) to go back
            Message::Back => action.add_instruction(HotelInstruction::Back),
        };

        action
    }

    const GLOBAL_SPACING: f32 = 6.;
    const GLOBAL_BUTTON_HEIGHT: f32 = 60.;
    const SQUAREBUTTONXY: f32 = 100.;

    /// Returns the view of the hotel screen
    pub fn view(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        // HEADER
        let header_row = self.view_header_row();

        widget::Column::new()
            .push(header_row)
            .spacing(spacing)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    //
    //  VIEW COMPOSING
    //

    /// Returns the view of the header row of the hotel screen
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

    //
    //  END OF VIEW COMPOSING
    //
}
