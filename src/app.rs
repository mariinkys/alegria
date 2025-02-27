// SPDX-License-Identifier: GPL-3.0-only

use iced::{Task, widget};

use crate::fl;

pub struct IcedAlegria {}

#[derive(Debug, Clone)]
pub enum Message {}

impl IcedAlegria {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        widget::Container::new(widget::Text::new(fl!("welcome"))).into()
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {}
        Task::none()
    }
}
