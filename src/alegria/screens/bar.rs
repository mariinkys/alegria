// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::{
    Alignment, Element, Length, Pixels, Task,
    widget::{self},
};
use sqlx::{Pool, Sqlite};

use crate::alegria::{
    action::AlegriaAction,
    core::models::{
        product::Product, product_category::ProductCategory, temporal_product::TemporalProduct,
        temporal_ticket::TemporalTicket,
    },
    utils::match_table_location_with_number,
};

#[derive(Default, Debug, Clone)]
pub enum TableLocation {
    #[default]
    Bar,
    Resturant,
    Garden,
}

/// We can identify a table using the table index and it's location
#[derive(Default, Debug, Clone)]
pub struct CurrentPositionState {
    /// Currently selected table location
    location: TableLocation,
    /// Currently selected table index
    table_index: usize,
}

pub struct Bar {
    /// Database of the application
    pub database: Option<Arc<Pool<Sqlite>>>,
    /// Product Categories
    product_categories: Vec<ProductCategory>,
    /// Selected product category products
    product_category_products: Option<Vec<Product>>,
    /// Currently selected table state
    currently_selected_pos_state: CurrentPositionState,
    /// Temporal Tickets hold the state of the maybe tickets of each table
    temporal_tickets_model: Vec<TemporalTicket>,
    // Keeps track of which product is selected in order to be able to modify it with the NumPad
    //selected_temporal_product: Option<TemporalProduct>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,

    FetchTemporalTickets,
    SetTemporalTickets(Vec<TemporalTicket>),

    FetchProductCategories,
    SetProductCategories(Vec<ProductCategory>),

    FetchProductCategoryProducts(Option<i32>),
    SetProductCategoryProducts(Option<Vec<Product>>),

    OnTableChange(CurrentPositionState),
    OnProductClicked(Option<i32>),
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
            currently_selected_pos_state: CurrentPositionState::default(),
            temporal_tickets_model: Vec::new(),
        }
    }

    /// Cleans the state of the bar screen preserving the database
    /// intended to be called when switching to another screen in order to save memory.
    pub fn clean_state(database: Option<Arc<Pool<Sqlite>>>) -> Self {
        Self {
            database,
            product_categories: Vec::new(),
            product_category_products: None,
            currently_selected_pos_state: CurrentPositionState::default(),
            temporal_tickets_model: Vec::new(),
        }
    }

    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<BarInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            Message::Back => action.add_instruction(BarInstruction::Back),

            Message::FetchTemporalTickets => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        TemporalTicket::get_all(pool.clone()),
                        |res| match res {
                            Ok(res) => Message::SetTemporalTickets(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::SetTemporalTickets(Vec::new())
                            }
                        },
                    ));
                }
            }
            Message::SetTemporalTickets(res) => {
                self.temporal_tickets_model = res;
            }

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

            Message::OnTableChange(table_state) => {
                self.currently_selected_pos_state = table_state;
                self.update(Message::FetchTemporalTickets);
            }

            Message::OnProductClicked(product_id) => {
                if let Some(new_product_id) = product_id {
                    if let Some(pool) = &self.database {
                        let temporal_ticket = TemporalTicket {
                            id: None,
                            table_id: self.currently_selected_pos_state.table_index as i32,
                            ticket_location: match_table_location_with_number(
                                self.currently_selected_pos_state.location.clone(),
                            ),
                            ticket_status: 0,
                            products: Vec::new(),
                        };

                        action.add_task(Task::perform(
                            TemporalTicket::upsert_ticket_by_id_and_tableloc(
                                pool.clone(),
                                temporal_ticket,
                                new_product_id,
                            ),
                            |res| match res {
                                Ok(_) => Message::FetchTemporalTickets,
                                Err(err) => {
                                    eprintln!("{err}");
                                    Message::SetProductCategoryProducts(None)
                                }
                            },
                        ));
                    }
                }
            }
        }

        action
    }

    /// Returns the view of the bar screen
    pub fn view(&self) -> Element<Message> {
        let header_row = self.view_header_row();
        let product_categories_container = self.view_product_categories_container();
        let product_category_products_container = self.view_product_category_products_container();

        let upper_left_row = widget::Row::new().push(self.view_tables_grid());

        let bottom_container = widget::Row::new()
            .push(upper_left_row)
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

    // Controls how many tables there are on a row
    const TABLES_PER_ROW: usize = 5;
    const NUMBER_OF_TABLES: usize = 30;

    /// Returns the view of the tables grid of the application
    fn view_tables_grid(&self) -> Element<Message> {
        let header = widget::Row::new().push(widget::Button::new("Bar"));

        let grid_spacing: f32 = 3.;
        let mut tables_grid = widget::Column::new().spacing(Pixels::from(grid_spacing));
        let mut current_row = widget::Row::new().spacing(Pixels::from(grid_spacing));
        for index in 0..Self::NUMBER_OF_TABLES {
            // TODO: Change button style depending on table status
            let table_button = widget::Button::new(
                widget::Text::new(format!("{}", index + 1))
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            )
            .width(Length::Fixed(40.))
            // TODO: Table location is now always bar
            .on_press(Message::OnTableChange(CurrentPositionState {
                location: TableLocation::Bar,
                table_index: index,
            }));
            current_row = current_row.push(table_button);

            if (index + 1) % Self::TABLES_PER_ROW == 0 {
                tables_grid = tables_grid.push(current_row);
                current_row = widget::Row::new().spacing(Pixels::from(grid_spacing));
            }
        }

        widget::Column::new().push(header).push(tables_grid).into()
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
                    .map(|product| {
                        widget::Button::new(product.name.as_str())
                            .on_press(Message::OnProductClicked(product.id))
                            .into()
                    })
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
