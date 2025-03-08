// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{Task, widget};
use sqlx::{Pool, Sqlite};

use crate::{
    alegria::screens::bar::{self, Bar},
    fl,
};

#[derive(Debug, Clone)]
pub enum Screen {
    Home,
    Bar,
}

pub struct IcedAlegria {
    /// Database of the application
    database: Option<Arc<Pool<Sqlite>>>,
    /// Represents a Screen of the App
    screen: Screen,
    /// Holds the state of the bar screen
    bar: Bar,
}

#[derive(Debug, Clone)]
pub enum Message {
    DatabaseLoaded(Arc<Pool<Sqlite>>),
    ChangeScreen(Screen),

    Bar(bar::Message),
}

impl IcedAlegria {
    pub fn new() -> Self {
        Self {
            database: None,
            screen: Screen::Home,
            bar: Bar::init(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let content = match self.screen {
            Screen::Home => widget::Button::new(widget::Text::new(fl!("welcome")))
                .on_press(Message::ChangeScreen(Screen::Bar))
                .into(),
            Screen::Bar => self.bar.view().map(Message::Bar),
        };

        widget::Container::new(content).into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        let mut tasks = vec![];

        match message {
            Message::DatabaseLoaded(pool) => {
                self.database = Some(pool);
                self.bar.database = self.database.clone();
            }
            Message::ChangeScreen(screen) => match screen {
                Screen::Home => {
                    self.screen = screen;
                    crate::alegria::screens::bar::Bar::clean_state(self.database.clone());
                }
                Screen::Bar => {
                    tasks.push(self.update(Message::Bar(bar::Message::FetchProductCategories)));
                    tasks.push(self.update(Message::Bar(bar::Message::FetchTemporalTickets)));
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

                for bar_task in action.instructions {
                    match bar_task {
                        bar::BarInstruction::Back => {
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
