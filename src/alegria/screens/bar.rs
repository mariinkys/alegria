// SPDX-License-Identifier: GPL-3.0-only

/// The main issue or dificulty with this page for now is the fact that everytime the user does something
/// we need to save it on the database and refresh, that way the data persists even if there is a power
/// shortage...
use std::sync::Arc;

use iced::{
    Alignment, Element, Length, Pixels, Task,
    widget::{self, text::LineHeight},
};
use sqlx::PgPool;
use sweeten::widget::text_input;

use crate::{
    alegria::{
        action::AlegriaAction,
        core::models::{
            product::Product, product_category::ProductCategory, temporal_product::TemporalProduct,
            temporal_ticket::TemporalTicket,
        },
        utils::{
            TemporalTicketStatus, match_number_with_temporal_ticket_status,
            match_table_location_with_number,
        },
    },
    fl,
};

/// Defines the different locations in which a table can be located at
#[derive(Default, Debug, Clone, PartialEq)]
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

/// What field of a TemporalProduct are we currently focusing on?
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalProductField {
    Quantity,
    Price,
}

/// Identifies an action of the numpad
#[derive(Debug, Clone, PartialEq)]
pub enum NumPadAction {
    Delete,
    Erase,
    Decimal,
}

/// Holds the pagination state (generic, for various entities)
#[derive(Debug, Clone)]
pub struct PaginationConfig {
    items_per_page: i32,
    current_page: i32,
}

/// Identifies a pagination action
#[derive(Debug, Clone, PartialEq)]
pub enum PaginationAction {
    Up,
    Down,
}

pub struct Bar {
    /// Database of the application
    pub database: Option<Arc<PgPool>>,
    /// Product Categories (for listing and then selecting products)
    product_categories: Vec<ProductCategory>,
    /// Selected product category products (if we clicked a category we will show it's products)
    product_category_products: Option<Vec<Product>>,
    /// Currently selected product_category id (needed for correct button styling)
    currently_selected_product_category: Option<i32>,
    /// Currently selected table state (helps us identify the currently selected table) TODO: Could we chnage this to an Option<TemporalTicket> and avoid this allotgether?
    currently_selected_pos_state: CurrentPositionState,
    /// Temporal Tickets hold the state of the maybe tickets of each table
    temporal_tickets_model: Vec<TemporalTicket>,
    /// Keeps track of which temporal product is active (within a temporal ticket) in order to be able to modify it with the NumPad
    active_temporal_product: Option<TemporalProduct>,
    /// Keeps track of which temporal product field is active (within a temporal product) in order to be able to modify it with the NumPad
    active_temporal_product_field: Option<TemporalProductField>,
    /// Holds the pagination state and config for the product categories list
    product_categories_pagination_state: PaginationConfig,
    /// Holds the pagination state and config for the product category products list
    product_category_products_pagination_state: PaginationConfig,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back

    FetchTemporalTickets, // Fetches all the current temporal tickets
    SetTemporalTickets(Vec<TemporalTicket>), // Sets the temporal tickets on the app state

    FetchProductCategories, // Fetches all the product categories
    SetProductCategories(Vec<ProductCategory>), // Sets the product categories on the state

    FetchProductCategoryProducts(Option<i32>), // Fetches the products for a given product category
    SetProductCategoryProducts(Option<Vec<Product>>), // Sets the products on the state

    OnTableChange(usize), // Callback after a table has been clicked
    ChangeCurrentTablesLocation(TableLocation), // Callback after we ask to change our current TableLocation
    OnProductClicked(Option<i32>), // When we click a product on the product list we have to add it to the temporal ticket...

    OnNumpadNumberClicked(u8), // Callback after a numpad number has been clicked
    OnNumpadKeyClicked(NumPadAction), // Callback after a numpad key (not a number) has been clicked

    FocusProductQuantity(TemporalProduct), // Callback after user focus the quantity field of a TemporalProduct
    FocusProductPrice(TemporalProduct), // Callback after user focus the price field of a TemporalProduct
    TemporalProductInput(TemporalProduct, String), // text_input of a temporal product

    ProductCategoriesPaginationAction(PaginationAction), // Try to go up or down a page on the ProductCategories
    ProductCategoryProductsPaginationAction(PaginationAction), // Try to go up or down a page on the ProductCategoryProducts
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
            currently_selected_product_category: None,
            currently_selected_pos_state: CurrentPositionState::default(),
            temporal_tickets_model: Vec::new(),
            active_temporal_product: None,
            active_temporal_product_field: None,
            // TODO: This should ideally come from a configfile (modifiable from another screen)
            product_categories_pagination_state: PaginationConfig {
                items_per_page: 13,
                current_page: 0,
            },
            product_category_products_pagination_state: PaginationConfig {
                items_per_page: 13,
                current_page: 0,
            },
        }
    }

    /// Cleans the state of the bar screen preserving the database
    /// intended to be called when switching to another screen in order to save memory.
    pub fn clean_state(database: Option<Arc<PgPool>>) -> Self {
        Self {
            database,
            product_categories: Vec::new(),
            product_category_products: None,
            currently_selected_product_category: None,
            currently_selected_pos_state: CurrentPositionState::default(),
            temporal_tickets_model: Vec::new(),
            active_temporal_product: None,
            active_temporal_product_field: None,
            product_categories_pagination_state: PaginationConfig {
                items_per_page: 13,
                current_page: 0,
            },
            product_category_products_pagination_state: PaginationConfig {
                items_per_page: 13,
                current_page: 0,
            },
        }
    }

    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<BarInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            // Asks the parent (app.rs) to go back
            Message::Back => action.add_instruction(BarInstruction::Back),

            // Fetches all the current temporal tickets
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
            // Sets the temporal tickets on the app state
            Message::SetTemporalTickets(res) => {
                self.temporal_tickets_model = res;

                // we need to update the active_temporal_product to so we can keep updating fields without having to focus again on the field
                // to update the active_temporal_product, also we want to keep the input of the text field of the currently selected
                // product, so we don't lose the '.' and we can input decimals
                if let Some(active_product) = &self.active_temporal_product {
                    let old_price_input = active_product.price_input.clone();
                    if let Some(product) = self
                        .temporal_tickets_model
                        .iter_mut()
                        .flat_map(|ticket| ticket.products.iter_mut())
                        .find(|product| product.id == active_product.id)
                    {
                        if self.active_temporal_product_field == Some(TemporalProductField::Price) {
                            product.price_input = old_price_input;
                        }
                        self.active_temporal_product = Some(product.clone());
                    }
                }
            }

            // Fetches all the product categories
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
            // Sets the product categories on the state
            Message::SetProductCategories(items) => {
                self.currently_selected_product_category = None;
                self.product_category_products = None;
                self.product_categories = items;
            }

            // Fetches the products for a given product category
            Message::FetchProductCategoryProducts(product_category_id) => {
                if let Some(pool) = &self.database {
                    self.currently_selected_product_category = product_category_id;
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
            // Sets the products on the state
            Message::SetProductCategoryProducts(items) => {
                self.product_category_products = items;
            }

            // Callback after a table has been clicked
            Message::OnTableChange(table_index) => {
                self.currently_selected_pos_state.table_index = table_index;
                self.active_temporal_product = None;
                self.active_temporal_product_field = None;
                self.update(Message::FetchTemporalTickets);
            }
            // Callback after we ask to change our current TableLocation
            Message::ChangeCurrentTablesLocation(location) => {
                self.currently_selected_pos_state.location = location;
            }

            // When we click a product on the product list we have to add it to the temporal ticket...
            Message::OnProductClicked(product_id) => {
                if let Some(new_product_id) = product_id {
                    if let Some(pool) = &self.database {
                        // Deselect the active temporal product
                        self.active_temporal_product = None;

                        let temporal_ticket = TemporalTicket {
                            id: None,
                            table_id: self.currently_selected_pos_state.table_index as i32,
                            ticket_location: match_table_location_with_number(
                                self.currently_selected_pos_state.location.clone(),
                            ),
                            ticket_status: 0,
                            products: Vec::new(),
                        };

                        // Upsert a temporal ticket with the clicked product
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

            // Callback after a numpad number has been clicked
            Message::OnNumpadNumberClicked(num) => {
                if let Some(product) = &self.active_temporal_product {
                    if let Some(field) = &self.active_temporal_product_field {
                        match field {
                            // we add the new number to the corresponding field and pass it as if it was inputed via the keyboard
                            // to the input handler
                            TemporalProductField::Quantity => {
                                let value = format!("{}{}", product.quantity, num);
                                return self
                                    .update(Message::TemporalProductInput(product.clone(), value));
                            }
                            TemporalProductField::Price => {
                                let value = format!("{}{}", product.price_input, num);
                                return self
                                    .update(Message::TemporalProductInput(product.clone(), value));
                            }
                        }
                    }
                }
            }
            // Callback after a numpad key (not a number) has been clicked
            Message::OnNumpadKeyClicked(action_type) => match action_type {
                // we clicked the delete button of the numpad
                NumPadAction::Delete => {
                    // we will need the current ticket to check if there are no more products we will need to delete the temporal ticket
                    let current_ticket = self.temporal_tickets_model.iter().find(|x| {
                        x.table_id == self.currently_selected_pos_state.table_index as i32
                            && x.ticket_location
                                == match_table_location_with_number(
                                    self.currently_selected_pos_state.location.clone(),
                                )
                    });

                    // if we have a product selected we delete that product
                    if let Some(active_product) = &self.active_temporal_product {
                        if let Some(pool) = &self.database {
                            action.add_task(Task::perform(
                                TemporalProduct::delete(
                                    pool.clone(),
                                    active_product.id.unwrap_or_default(),
                                ),
                                |res| match res {
                                    Ok(_) => Message::FetchTemporalTickets,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::FetchTemporalTickets
                                    }
                                },
                            ));

                            self.active_temporal_product = None;
                            self.active_temporal_product_field = None;

                            // check if there are no more products we will need to delete the temporal ticket
                            if let Some(ticket) = current_ticket {
                                if ticket.products.len() == 1 {
                                    action.add_task(Task::perform(
                                        TemporalTicket::delete(
                                            pool.clone(),
                                            ticket.id.unwrap_or_default(),
                                        ),
                                        |res| match res {
                                            Ok(_) => Message::FetchTemporalTickets,
                                            Err(err) => {
                                                eprintln!("{err}");
                                                Message::FetchTemporalTickets
                                            }
                                        },
                                    ));
                                }
                            }
                        }
                    // if we don't have a product selected but there is a ticket and we pressed delete
                    } else if let Some(ticket) = current_ticket {
                        if let Some(product) = ticket.products.first() {
                            if let Some(pool) = &self.database {
                                // we delete the first product of the list
                                action.add_task(Task::perform(
                                    TemporalProduct::delete(
                                        pool.clone(),
                                        product.id.unwrap_or_default(),
                                    ),
                                    |res| match res {
                                        Ok(_) => Message::FetchTemporalTickets,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::FetchTemporalTickets
                                        }
                                    },
                                ));

                                // check if there are no more products we will need to delete the temporal ticket
                                if ticket.products.len() == 1 {
                                    action.add_task(Task::perform(
                                        TemporalTicket::delete(
                                            pool.clone(),
                                            ticket.id.unwrap_or_default(),
                                        ),
                                        |res| match res {
                                            Ok(_) => Message::FetchTemporalTickets,
                                            Err(err) => {
                                                eprintln!("{err}");
                                                Message::FetchTemporalTickets
                                            }
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }
                // we clicked the erase button of the numpad
                NumPadAction::Erase => {
                    if let Some(product) = &self.active_temporal_product {
                        if let Some(field) = &self.active_temporal_product_field {
                            match field {
                                // we substract a char of the corresponding field and pass it to the
                                // input update function as if it was inputed via keyboard
                                TemporalProductField::Quantity => {
                                    let product_quantity = product.quantity.to_string();
                                    if product_quantity.len() > 1 {
                                        let value = &product_quantity[..product_quantity.len() - 1];
                                        return self.update(Message::TemporalProductInput(
                                            product.clone(),
                                            value.to_string(),
                                        ));
                                    } else {
                                        // if we only have one "char" we put a 0
                                        return self.update(Message::TemporalProductInput(
                                            product.clone(),
                                            0.to_string(),
                                        ));
                                    }
                                }
                                TemporalProductField::Price => {
                                    let product_price = &product.price_input;
                                    if product_price.len() > 1 {
                                        let value = &product_price[..product_price.len() - 1];

                                        return self.update(Message::TemporalProductInput(
                                            product.clone(),
                                            value.to_string(),
                                        ));
                                    } else {
                                        return self.update(Message::TemporalProductInput(
                                            product.clone(),
                                            String::new(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                // we clicked the '.' button of the numpad
                NumPadAction::Decimal => {
                    if let Some(product) = &self.active_temporal_product {
                        if let Some(field) = &self.active_temporal_product_field {
                            // only the price can be decimal
                            if *field == TemporalProductField::Price {
                                return self.update(Message::TemporalProductInput(
                                    product.clone(),
                                    format!("{}.", product.price_input),
                                ));
                            }
                        }
                    }
                }
            },

            // Callback after user focus the quantity field of a TemporalProduct
            Message::FocusProductQuantity(product) => {
                self.active_temporal_product = Some(product);
                self.active_temporal_product_field = Some(TemporalProductField::Quantity);
            }
            // Callback after user focus the price field of a TemporalProduct
            Message::FocusProductPrice(product) => {
                self.active_temporal_product = Some(product);
                self.active_temporal_product_field = Some(TemporalProductField::Price);
            }
            // text_input of a temporal product
            Message::TemporalProductInput(product, new_value) => {
                if let Some(field) = &self.active_temporal_product_field {
                    let mut mutable_product = product;

                    match field {
                        TemporalProductField::Quantity => {
                            // if we are focusing the quantity we assign the new_value
                            if let Ok(num) = new_value.parse::<i32>() {
                                mutable_product.quantity = num;
                            } else if new_value.is_empty() {
                                mutable_product.quantity = 0;
                            }
                        }
                        TemporalProductField::Price => {
                            //let new_value = new_value.trim_start_matches('0').to_string();
                            // We ignore the input if we already have two decimals and we're trying to add more
                            let ignore_action = new_value.len() > mutable_product.price_input.len()
                                && mutable_product
                                    .price_input
                                    .find('.')
                                    .is_some_and(|idx| mutable_product.price_input.len() - idx > 2);

                            if !ignore_action {
                                if let Ok(num) = new_value.parse::<f32>() {
                                    mutable_product.price = Some(num);

                                    if let Some(active_product) = &mut self.active_temporal_product
                                    {
                                        active_product.price_input = new_value;
                                    }
                                } else if new_value.is_empty() {
                                    mutable_product.price = Some(0.0);

                                    if let Some(active_product) = &mut self.active_temporal_product
                                    {
                                        active_product.price_input = new_value;
                                    }
                                }
                            }
                        }
                    }

                    if let Some(pool) = &self.database {
                        action.add_task(Task::perform(
                            TemporalProduct::edit(pool.clone(), mutable_product),
                            |res| match res {
                                Ok(_) => Message::FetchTemporalTickets,
                                Err(err) => {
                                    eprintln!("{err}");
                                    Message::FetchTemporalTickets
                                }
                            },
                        ));
                    }
                }
            }

            // Try to go up or down a page on the ProductCategories
            Message::ProductCategoriesPaginationAction(action) => match action {
                PaginationAction::Up => {
                    if self.product_categories_pagination_state.current_page > 0 {
                        self.product_categories_pagination_state.current_page -= 1;
                    }
                }
                PaginationAction::Down => {
                    let next_page_start = (self.product_categories_pagination_state.current_page
                        + 1)
                        * self.product_categories_pagination_state.items_per_page;
                    if next_page_start
                        < self.product_categories.len().try_into().unwrap_or_default()
                    {
                        self.product_categories_pagination_state.current_page += 1;
                    }
                }
            },
            // Try to go up or down a page on the ProductCategoryProducts
            Message::ProductCategoryProductsPaginationAction(action) => match action {
                PaginationAction::Up => {
                    if self.product_category_products_pagination_state.current_page > 0 {
                        self.product_category_products_pagination_state.current_page -= 1;
                    }
                }
                PaginationAction::Down => {
                    let next_page_start =
                        (self.product_category_products_pagination_state.current_page + 1)
                            * self
                                .product_category_products_pagination_state
                                .items_per_page;
                    if next_page_start
                        < self
                            .product_category_products
                            .as_ref()
                            .map(|v| v.len())
                            .unwrap_or(0)
                            .try_into()
                            .unwrap_or_default()
                    {
                        self.product_category_products_pagination_state.current_page += 1;
                    }
                }
            },
        }

        action
    }

    const GLOBAL_SPACING: f32 = 6.;
    const GLOBAL_BUTTON_HEIGHT: f32 = 60.;

    /// Returns the view of the bar screen
    pub fn view(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        // HEADER
        let header_row = self.view_header_row();

        // BOTTOM RIGHT SIDE
        // TODO: Pagination
        let product_categories_container = self.view_product_categories_container();
        let product_category_products_container = self.view_product_category_products_container();
        let right_side_container = widget::Row::new()
            .push(product_categories_container)
            .push(product_category_products_container)
            .spacing(spacing)
            .width(Length::Fill);

        // BOTTOM LEFT SIDE
        let left_side_upper_row_left_col = self.view_tables_grid();
        let left_side_upper_row_right_col = widget::Column::new()
            .push(self.view_current_ticket_total_price())
            .push(self.view_numpad())
            .width(Length::Fixed(235.)) //TODO: Maybe this should not be like this but the custom widget also gives some trouble
            .spacing(spacing);
        let left_side_upper_row = widget::Row::new()
            .push(left_side_upper_row_left_col)
            .push(left_side_upper_row_right_col)
            .align_y(Alignment::Center)
            .spacing(spacing);
        let left_side_down_row = self.view_current_ticket_products();
        let left_side_container = widget::Column::new()
            .push(left_side_upper_row)
            .push(left_side_down_row)
            .spacing(spacing)
            .width(Length::Fill);

        let bottom_row = widget::Row::new()
            .push(left_side_container)
            .push(right_side_container)
            .spacing(spacing);

        widget::Column::new()
            .push(header_row)
            .push(bottom_row)
            .spacing(spacing)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    //
    //  VIEW COMPOSING
    //

    const TITLE_TEXT_SIZE: f32 = 25.0;

    /// Returns the view of the header row of the bar screen
    fn view_header_row(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
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
            .push(
                widget::Text::new(fl!("bar"))
                    .size(Pixels::from(Self::TITLE_TEXT_SIZE))
                    .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .spacing(spacing)
            .into()
    }

    // Controls how many tables there are on a row
    const TABLES_PER_ROW: usize = 5;
    const NUMBER_OF_TABLES: usize = 30;

    /// Returns the view of the tables grid of the application
    fn view_tables_grid(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        let header = widget::Row::new()
            .push(
                widget::Button::new(
                    widget::Text::new(fl!("bar"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ChangeCurrentTablesLocation(TableLocation::Bar))
                .style(|t, s| self.determine_location_button_color(t, s, TableLocation::Bar))
                .height(button_height)
                .width(Length::Fill),
            )
            .push(
                widget::Button::new(
                    widget::Text::new(fl!("restaurant"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ChangeCurrentTablesLocation(
                    TableLocation::Resturant,
                ))
                .style(|t, s| self.determine_location_button_color(t, s, TableLocation::Resturant))
                .height(button_height)
                .width(Length::Fill),
            )
            .push(
                widget::Button::new(
                    widget::Text::new(fl!("garden"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ChangeCurrentTablesLocation(TableLocation::Garden))
                .style(|t, s| self.determine_location_button_color(t, s, TableLocation::Garden))
                .height(button_height)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .spacing(spacing);

        let mut tables_grid = widget::Column::new().spacing(spacing).width(Length::Fill);
        let mut current_row = widget::Row::new().spacing(spacing).width(Length::Fill);
        for index in 0..Self::NUMBER_OF_TABLES {
            let table_button = widget::Button::new(
                widget::Text::new(format!("{}", index + 1))
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .height(button_height)
            .style(move |t, s| self.determine_table_button_color(t, s, index))
            .on_press(Message::OnTableChange(index));
            current_row = current_row.push(table_button);

            if (index + 1) % Self::TABLES_PER_ROW == 0 {
                tables_grid = tables_grid.push(current_row);
                current_row = widget::Row::new().spacing(spacing).width(Length::Fill);
            }
        }

        widget::Column::new()
            .push(header)
            .push(tables_grid)
            .width(Length::Fill)
            .spacing(spacing)
            .into()
    }

    /// Returns the view of the product categories of the bar screen
    fn view_product_categories_container(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        // Calculate the indices for the current page
        let start_index: usize = self.product_categories_pagination_state.current_page as usize
            * self.product_categories_pagination_state.items_per_page as usize;
        let end_index = usize::min(
            start_index + self.product_categories_pagination_state.items_per_page as usize,
            self.product_categories.len(),
        );

        let categories_buttons: Vec<_> = self.product_categories[start_index..end_index]
            .iter()
            .map(|category| {
                widget::Button::new(
                    widget::Text::new(category.name.as_str())
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::FetchProductCategoryProducts(category.id))
                .style(move |t, s| self.determine_product_category_button_color(t, s, category.id))
                .height(button_height)
                .width(Length::Fill)
                .into()
            })
            .collect();
        let categories_col = widget::Column::with_children(categories_buttons)
            .spacing(spacing)
            .height(Length::Fill);

        let pagination_buttons = widget::Row::new()
            .push(
                widget::Button::new(
                    widget::Text::new(fl!("up"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ProductCategoriesPaginationAction(
                    PaginationAction::Up,
                ))
                .height(button_height)
                .width(Length::Fill),
            )
            .push(
                widget::Button::new(
                    widget::Text::new(fl!("down"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ProductCategoriesPaginationAction(
                    PaginationAction::Down,
                ))
                .height(button_height)
                .width(Length::Fill),
            )
            .spacing(spacing)
            .height(Length::Shrink);

        let result_column = widget::Column::new()
            .push(categories_col)
            .push(pagination_buttons)
            .height(Length::Fill);

        widget::Container::new(result_column)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    /// Returns the view of the currently selected product category products of the bar screen
    fn view_product_category_products_container(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        // Calculate the indices for the current page
        let start_index: usize = self.product_category_products_pagination_state.current_page
            as usize
            * self
                .product_category_products_pagination_state
                .items_per_page as usize;
        let end_index = usize::min(
            start_index
                + self
                    .product_category_products_pagination_state
                    .items_per_page as usize,
            self.product_category_products
                .as_ref()
                .map(|v| v.len())
                .unwrap_or(0),
        );

        let products_buttons: Vec<_> = self
            .product_category_products
            .as_ref()
            .map(|products| {
                products[start_index..end_index]
                    .iter()
                    .map(|product| {
                        widget::Button::new(
                            widget::Text::new(product.name.as_str())
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::OnProductClicked(product.id))
                        .height(button_height)
                        .width(Length::Fill)
                        .into()
                    })
                    .collect()
            })
            .unwrap_or_default();
        let products_col = widget::Column::with_children(products_buttons)
            .spacing(spacing)
            .height(Length::Fill);

        let pagination_buttons = widget::Row::new()
            .push(
                widget::Button::new(
                    widget::Text::new(fl!("up"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ProductCategoryProductsPaginationAction(
                    PaginationAction::Up,
                ))
                .height(button_height)
                .width(Length::Fill),
            )
            .push(
                widget::Button::new(
                    widget::Text::new(fl!("down"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ProductCategoryProductsPaginationAction(
                    PaginationAction::Down,
                ))
                .height(button_height)
                .width(Length::Fill),
            )
            .spacing(spacing)
            .height(Length::Shrink);

        let result_column = widget::Column::new()
            .push(products_col)
            .push(pagination_buttons)
            .height(Length::Fill);

        widget::Container::new(result_column)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    /// Returns the view of the product (list) of the currently selected ticket
    fn view_current_ticket_products(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        // TODO: We could do this OnTableClick and save the Option<TemporalTicket> on state and do not search for it here and on the colors functions
        let current_ticket = &self.temporal_tickets_model.iter().find(|x| {
            x.ticket_location
                == match_table_location_with_number(
                    self.currently_selected_pos_state.location.clone(),
                )
                && x.table_id == self.currently_selected_pos_state.table_index as i32
        });

        if current_ticket.is_some() {
            let mut products_column = widget::Column::new().spacing(spacing);

            for product in &current_ticket.unwrap().products {
                let product_quantity_str = product.quantity.to_string();

                let product_row = widget::Row::new()
                    .push(
                        widget::Text::new(&product.name)
                            .size(Pixels::from(25.))
                            .width(Length::Fill)
                            .wrapping(widget::text::Wrapping::None),
                    )
                    .push(
                        text_input(&product_quantity_str, &product_quantity_str)
                            .on_focus(move |_| Message::FocusProductQuantity(product.clone()))
                            .on_input(|value| Message::TemporalProductInput(product.clone(), value))
                            .size(Pixels::from(25.)),
                    )
                    .push(
                        text_input(&product.price_input, &product.price_input)
                            .on_focus(move |_| Message::FocusProductPrice(product.clone()))
                            .on_input(|value| Message::TemporalProductInput(product.clone(), value))
                            .size(Pixels::from(25.)),
                    )
                    .spacing(spacing)
                    .align_y(Alignment::Center);

                products_column = products_column.push(product_row);
            }

            widget::Scrollable::new(products_column).into()
        } else {
            widget::Row::new()
                .push(
                    widget::Text::new(fl!("no-products"))
                        .size(Pixels::from(25.))
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                )
                .width(Length::Fill)
                .into()
        }
    }

    /// Returns the view of the numpad
    fn view_numpad(&self) -> Element<Message> {
        crate::alegria::widgets::numpad::Numpad::new()
            .on_number_clicked(Message::OnNumpadNumberClicked)
            .on_back_clicked(Message::OnNumpadKeyClicked(NumPadAction::Erase))
            .on_delete_clicked(Message::OnNumpadKeyClicked(NumPadAction::Delete))
            .on_comma_clicked(Message::OnNumpadKeyClicked(NumPadAction::Decimal))
            .into()
    }

    /// Returns the view of the product (list) of the currently selected ticket
    fn view_current_ticket_total_price(&self) -> Element<Message> {
        // TODO: We could do this OnTableClick and save the Option<TemporalTicket> on state and do not search for it here and on the colors functions
        let current_ticket = &self.temporal_tickets_model.iter().find(|x| {
            x.ticket_location
                == match_table_location_with_number(
                    self.currently_selected_pos_state.location.clone(),
                )
                && x.table_id == self.currently_selected_pos_state.table_index as i32
        });

        let text = if let Some(ticket) = current_ticket {
            let mut price = 0.;
            for product in &ticket.products {
                for _ in 0..product.quantity {
                    price += product.price.unwrap_or(0.);
                }
            }

            widget::Text::new(format!("{:.2}", price))
                .size(Pixels::from(25.))
                .line_height(LineHeight::Relative(2.))
        } else {
            widget::Text::new(fl!("unknown"))
                .size(Pixels::from(25.))
                .line_height(LineHeight::Relative(2.))
        };

        widget::Container::new(text)
            .style(widget::container::bordered_box)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    }

    //
    //  END OF VIEW COMPOSING
    //

    //
    // HELPERS
    //

    /// Determines the color a button of the tables grid should be given the table index, using the temporal_tickets model
    fn determine_table_button_color(
        &self,
        t: &iced::Theme,
        s: widget::button::Status,
        t_id: usize,
    ) -> widget::button::Style {
        let table_id = t_id as i32;

        // We have it currently selected
        if self.currently_selected_pos_state.table_index as i32 == table_id {
            match s {
                widget::button::Status::Hovered => {
                    return widget::button::primary(t, widget::button::Status::Hovered);
                }
                _ => {
                    return widget::button::primary(t, widget::button::Status::Active);
                }
            }
        }

        let current_ticket = self.temporal_tickets_model.iter().find(|x| {
            x.table_id == table_id
                && x.ticket_location
                    == match_table_location_with_number(
                        self.currently_selected_pos_state.location.clone(),
                    )
        });

        // there is not ticket on this table
        if current_ticket.is_none() {
            match s {
                widget::button::Status::Hovered => {
                    return widget::button::secondary(t, widget::button::Status::Hovered);
                }
                _ => return widget::button::secondary(t, widget::button::Status::Active),
            }

        // there is a pending ticket on this table (we are not currently selecting this ticket)
        } else if current_ticket.is_some_and(|y| {
            match_number_with_temporal_ticket_status(y.ticket_status)
                == TemporalTicketStatus::Pending
        }) {
            match s {
                widget::button::Status::Hovered => {
                    return widget::button::danger(t, widget::button::Status::Hovered);
                }
                _ => return widget::button::danger(t, widget::button::Status::Active),
            }

        // there is a printed ticket on this table (we are not currently selecting this ticket)
        } else if current_ticket.is_some_and(|y| {
            match_number_with_temporal_ticket_status(y.ticket_status)
                == TemporalTicketStatus::Printed
        }) {
            match s {
                widget::button::Status::Hovered => {
                    return widget::button::success(t, widget::button::Status::Hovered);
                }
                _ => return widget::button::success(t, widget::button::Status::Active),
            }
        }

        widget::button::secondary(t, widget::button::Status::Disabled)
    }

    /// Determines the color of the locations buttons using the current location of the state and given which location is which one
    fn determine_location_button_color(
        &self,
        t: &iced::Theme,
        s: widget::button::Status,
        loc: TableLocation,
    ) -> widget::button::Style {
        // we are currently in this location
        if loc == self.currently_selected_pos_state.location {
            match s {
                widget::button::Status::Hovered => {
                    widget::button::primary(t, widget::button::Status::Hovered)
                }
                _ => widget::button::primary(t, widget::button::Status::Active),
            }
        } else {
            match s {
                widget::button::Status::Hovered => {
                    widget::button::secondary(t, widget::button::Status::Hovered)
                }
                _ => widget::button::secondary(t, widget::button::Status::Active),
            }
        }
    }

    /// Determines the color a button of the tables grid should be given the table index, using the temporal_tickets model
    fn determine_product_category_button_color(
        &self,
        t: &iced::Theme,
        s: widget::button::Status,
        cat_id: Option<i32>,
    ) -> widget::button::Style {
        // we are currently selecting this category
        if self.currently_selected_product_category == cat_id {
            match s {
                widget::button::Status::Hovered => {
                    widget::button::primary(t, widget::button::Status::Hovered)
                }
                _ => widget::button::primary(t, widget::button::Status::Active),
            }
        } else {
            match s {
                widget::button::Status::Hovered => {
                    widget::button::secondary(t, widget::button::Status::Hovered)
                }
                _ => widget::button::secondary(t, widget::button::Status::Active),
            }
        }
    }

    //
    //  END OF HELPERS
    //
}
