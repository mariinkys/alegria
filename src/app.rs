// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{
    Alignment, Length, Task,
    widget::{Column, Row, button, container, text},
};
use sqlx::PgPool;

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

#[derive(Debug, Clone, PartialEq)]
pub enum LoadState {
    Loading,
    Loaded,
}

pub struct IcedAlegria {
    /// Database of the application
    database: Option<Arc<PgPool>>,
    /// Load Status (don't show the main menu if db has not connected)
    load_state: LoadState,
    /// Represents a Screen of the App
    screen: Screen,
    /// Holds the state of the bar screen
    bar: Bar,
    /// Holds the state of the hotel screen
    hotel: Hotel,
}

#[derive(Debug, Clone)]
pub enum Message {
    DatabaseLoaded(Result<Arc<PgPool>, String>),
    ChangeScreen(Screen),

    Bar(bar::Message),
    Hotel(hotel::Message),
}

impl IcedAlegria {
    pub fn new() -> Self {
        Self {
            database: None,
            screen: Screen::Home,
            bar: Bar::default(),
            hotel: Hotel::default(),
            load_state: LoadState::Loading,
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        if self.load_state == LoadState::Loading {
            return container(text(fl!("loading")))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into();
        }

        let content = match self.screen {
            Screen::Home => {
                let buttons_row = Row::new()
                    .push(
                        button(
                            text(fl!("bar"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::ChangeScreen(Screen::Bar))
                        .width(100.)
                        .height(100.),
                    )
                    .push(
                        button(
                            text(fl!("hotel"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::ChangeScreen(Screen::Hotel))
                        .width(100.)
                        .height(100.),
                    )
                    .push(
                        button(
                            text(fl!("managment"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .width(100.)
                        .height(100.),
                    )
                    .spacing(5.)
                    .height(Length::Shrink);

                let centered_buttons = container(buttons_row)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .height(Length::Fill)
                    .align_y(Alignment::Center);

                let app_text = text("dev.mariinkys.Alegría dev-0.1.0")
                    .align_x(Alignment::End)
                    .width(Length::Fill);

                let content = Column::new()
                    .push(centered_buttons)
                    .push(app_text)
                    .width(Length::Fill)
                    .height(Length::Fill);

                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            Screen::Bar => self.bar.view().map(Message::Bar),
            Screen::Hotel => self.hotel.view().map(Message::Hotel),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(8.)
            .into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        let mut tasks = vec![];

        match message {
            Message::DatabaseLoaded(db_res) => match db_res {
                Ok(pool) => {
                    self.database = Some(pool);
                    self.load_state = LoadState::Loaded;
                }
                Err(err) => {
                    eprintln!("Database init failed: {}", err);
                    std::process::exit(1);
                }
            },
            Message::ChangeScreen(screen) => match screen {
                Screen::Home => {
                    self.screen = screen;
                    self.bar = bar::Bar::default();
                    self.hotel = hotel::Hotel::default();
                }
                Screen::Bar => {
                    self.bar.database = self.database.clone();
                    tasks.push(self.update(Message::Bar(bar::Message::InitPage)));
                    self.screen = screen;
                }
                Screen::Hotel => {
                    self.hotel.database = self.database.clone();
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
