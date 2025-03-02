// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{
    Element, Task,
    widget::{self, Row},
};
use sqlx::{Pool, Sqlite};

use crate::alegria::{
    action::AlegriaAction,
    core::models::{product::Product, product_category::ProductCategory},
};

pub struct Bar {
    /// Database of the application
    pub database: Option<Arc<Pool<Sqlite>>>,
    /// Product Categories
    product_categories: Vec<ProductCategory>,
    /// Selected product category products
    product_category_products: Option<Vec<Product>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,

    FetchProductCategories,
    SetProductCategories(Vec<ProductCategory>),

    FetchProductCategoryProducts(Option<i32>),
    SetProductCategoryProducts(Option<Vec<Product>>),
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum BarInstruction {
    Back,
}

impl Bar {
    /// Initializes the bar screen
    pub fn init() -> Self {
        Self {
            database: None,
            product_categories: Vec::new(),
            product_category_products: None,
        }
    }

    /// Handles messages emitted by the application and its widgets.
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
            Message::FetchProductCategoryProducts(product_category_id) => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        Product::get_all_by_category(
                            pool.clone(),
                            product_category_id.unwrap_or_default(),
                        ),
                        |res| match res {
                            Ok(items) => Message::SetProductCategoryProducts(Some(items)),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetProductCategoryProducts(None)
                            }
                        },
                    ));
                }
            }
            Message::SetProductCategoryProducts(items) => {
                self.product_category_products = items;
            }
        }

        action
    }

    /// Returns the view of the bar screen
    pub fn view(&self) -> Element<Message> {
        let header_row = self.view_header_row();
        let product_categories_container = self.view_product_categories_container();
        let product_category_products_container = self.view_product_category_products_container();

        let bottom_container = Row::new()
            .push(product_categories_container)
            .push(product_category_products_container);

        widget::Column::new()
            .push(header_row)
            .push(bottom_container)
            .into()
    }

    //
    //  VIEW COMPOSING
    //

    /// Returns the view of the header row of the bar screen
    fn view_header_row(&self) -> Element<Message> {
        let back_button = widget::Button::new("Back").on_press(Message::Back);

        widget::Row::new().push(back_button).into()
    }

    /// Returns the view of the product categories of the bar screen
    fn view_product_categories_container(&self) -> Element<Message> {
        let categories_buttons: Vec<_> = self
            .product_categories
            .iter()
            .map(|category| {
                widget::Button::new(category.name.as_str())
                    .on_press(Message::FetchProductCategoryProducts(category.id))
                    .into()
            })
            .collect();
        let categories_col = widget::Column::with_children(categories_buttons);

        widget::Container::new(categories_col).into()
    }

    /// Returns the view of the currently selected product category products of the bar screen
    fn view_product_category_products_container(&self) -> Element<Message> {
        let products_buttons: Vec<_> = self
            .product_category_products
            .as_ref()
            .map(|products| {
                products
                    .iter()
                    .map(|product| widget::Button::new(product.name.as_str()).into())
                    .collect()
            })
            .unwrap_or_default();
        let products_col = widget::Column::with_children(products_buttons);

        widget::Container::new(products_col).into()
    }

    //
    //  END OF VIEW COMPOSING
    //
}
