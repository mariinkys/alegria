// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{Element, Task, widget};
use sqlx::{Pool, Sqlite};

use crate::alegria::core::models::product_category::ProductCategory;

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
pub enum BarTasks {
    Back,
}

impl Bar {
    pub fn init() -> Self {
        Self {
            database: None,
            product_categories: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> (Vec<Task<Message>>, Vec<BarTasks>) {
        let mut tasks = Vec::new();
        let mut bar_tasks = Vec::new();

        match message {
            Message::Back => bar_tasks.push(BarTasks::Back),
            Message::FetchProductCategories => {
                if let Some(pool) = &self.database {
                    tasks.push(Task::perform(
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

        (tasks, bar_tasks)
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
