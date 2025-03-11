// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{Alignment, Length, Padding, Pixels, Task, widget};
use sqlx::{Pool, Sqlite};

use crate::{
    alegria::screens::{
        bar::{self, Bar},
        hotel::{self, Hotel},
    },
    fl,
};

#[derive(Debug, Clone)]
pub enum Screen {
    Home,
    Bar,
    Hotel,
}

pub struct IcedAlegria {
    /// Database of the application
    database: Option<Arc<Pool<Sqlite>>>,
    /// Represents a Screen of the App
    screen: Screen,
    /// Holds the state of the bar screen
    bar: Bar,
    /// Holds the state of the hotel screen
    hotel: Hotel,
}

#[derive(Debug, Clone)]
pub enum Message {
    DatabaseLoaded(Arc<Pool<Sqlite>>),
    ChangeScreen(Screen),

    Bar(bar::Message),
    Hotel(hotel::Message),
}

impl IcedAlegria {
    pub fn new() -> Self {
        Self {
            database: None,
            screen: Screen::Home,
            bar: Bar::init(),
            hotel: Hotel::init(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let content = match self.screen {
            Screen::Home => {
                let buttons_row = widget::Row::new()
                    .push(
                        widget::Button::new(
                            widget::Text::new(fl!("bar"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::ChangeScreen(Screen::Bar))
                        .width(Length::Fixed(100.))
                        .height(Length::Fixed(100.)),
                    )
                    .push(
                        widget::Button::new(
                            widget::Text::new(fl!("hotel"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::ChangeScreen(Screen::Hotel))
                        .width(Length::Fixed(100.))
                        .height(Length::Fixed(100.)),
                    )
                    .push(
                        widget::Button::new(
                            widget::Text::new(fl!("managment"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .width(Length::Fixed(100.))
                        .height(Length::Fixed(100.)),
                    )
                    .spacing(Pixels::from(5.));

                widget::Container::new(buttons_row)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            Screen::Bar => self.bar.view().map(Message::Bar),
            Screen::Hotel => self.hotel.view().map(Message::Hotel),
        };

        widget::Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::new(8.))
            .into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        let mut tasks = vec![];

        match message {
            Message::DatabaseLoaded(pool) => {
                self.database = Some(pool);
                self.bar.database = self.database.clone();
                self.hotel.set_database(self.database.clone());
            }
            Message::ChangeScreen(screen) => match screen {
                Screen::Home => {
                    self.screen = screen;
                    self.bar =
                        crate::alegria::screens::bar::Bar::clean_state(self.database.clone());
                    self.hotel =
                        crate::alegria::screens::hotel::Hotel::clean_state(self.database.clone());
                }
                Screen::Bar => {
                    tasks.push(self.update(Message::Bar(bar::Message::FetchProductCategories)));
                    tasks.push(self.update(Message::Bar(bar::Message::FetchTemporalTickets)));
                    self.screen = screen;
                }
                Screen::Hotel => {
                    self.screen = screen;
                }
            },
            Message::Bar(message) => {
                let action = self.bar.update(message);
                // TODO: Can I abstract this into action?
                let bar_tasks: Vec<Task<Message>> = action
                    .tasks
                    .into_iter()
                    .map(|task| task.map(Message::Bar))
                    .collect();
                tasks.extend(bar_tasks);

                for bar_instruction in action.instructions {
                    match bar_instruction {
                        bar::BarInstruction::Back => {
                            let _ = self.update(Message::ChangeScreen(Screen::Home));
                        }
                    }
                }
            }
            Message::Hotel(message) => {
                let action = self.hotel.update(message);
                // TODO: Can I abstract this into action?
                let hotel_tasks: Vec<Task<Message>> = action
                    .tasks
                    .into_iter()
                    .map(|task| task.map(Message::Hotel))
                    .collect();
                tasks.extend(hotel_tasks);

                for hotel_instruction in action.instructions {
                    match hotel_instruction {
                        hotel::HotelInstruction::Back => {
                            let _ = self.update(Message::ChangeScreen(Screen::Home));
                        }
                    }
                }
            }
        }

        Task::batch(tasks)
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::GruvboxLight
    }
}
