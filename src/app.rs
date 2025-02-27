// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{Task, widget};
use sqlx::{Pool, Sqlite};

use crate::fl;

pub struct IcedAlegria {
    /// Database of the application
    database: Option<Arc<Pool<Sqlite>>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    DatabaseLoaded(Arc<Pool<Sqlite>>),
}

impl IcedAlegria {
    pub fn new() -> Self {
        Self { database: None }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        widget::Container::new(widget::Text::new(fl!("welcome"))).into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::DatabaseLoaded(pool) => {
                self.database = Some(pool);
            }
        }
        Task::none()
    }
}
