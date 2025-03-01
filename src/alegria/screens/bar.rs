// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{Element, Task, widget};
use sqlx::{Pool, Sqlite};

use crate::alegria::{action::AlegriaAction, core::models::product_category::ProductCategory};

pub struct Bar {
    /// Database of the application
    pub database: Option<Arc<Pool<Sqlite>>>,
    /// Product Categories
    product_categories: Vec<ProductCategory>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,

    FetchProductCategories,
    SetProductCategories(Vec<ProductCategory>),
}

// Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum BarInstruction {
    Back,
}

impl Bar {
    pub fn init() -> Self {
        Self {
            database: None,
            product_categories: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> AlegriaAction<BarInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            Message::Back => action.add_instruction(BarInstruction::Back),
            Message::FetchProductCategories => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        ProductCategory::get_all(pool.clone()),
                        |res| match res {
                            Ok(items) => Message::SetProductCategories(items),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetProductCategories(Vec::new())
                            }
                        },
                    ));
                }
            }
            Message::SetProductCategories(items) => {
                self.product_categories = items;
            }
        }

        action
    }

    pub fn view(&self) -> Element<Message> {
        let back_button = widget::Button::new("Back").on_press(Message::Back);

        let categories_buttons: Vec<_> = self
            .product_categories
            .iter()
            .map(|category| widget::Button::new(category.name.as_str()).into())
            .collect();

        let categories_col = widget::Column::with_children(categories_buttons);

        widget::Column::new()
            .push(back_button)
            .push(categories_col)
            .into()
    }
}
