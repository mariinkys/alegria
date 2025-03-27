// SPDX-License-Identifier: GPL-3.0-only

/// The main issue or dificulty with this page for now is the fact that everytime the user does something
/// we need to save it on the database and refresh, that way the data persists even if there is a power
/// shortage...
use std::sync::Arc;

use iced::{
    Alignment, Element, Length, Pixels, Task,
    widget::{
        self, Column, Row, Scrollable, Space, button, column, container, pick_list, row, text,
    },
};
use sqlx::PgPool;
use sweeten::widget::text_input;

use crate::{
    alegria::{
        action::AlegriaAction,
        core::{
            models::{
                product::Product, product_category::ProductCategory, simple_invoice::SimpleInvoice,
                temporal_product::TemporalProduct, temporal_ticket::TemporalTicket,
            },
            print::AlegriaPrinter,
        },
        utils::{
            TemporalTicketStatus, match_number_with_temporal_ticket_status,
            match_table_location_with_number,
        },
        widgets::modal::modal,
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

impl Default for PaginationConfig {
    fn default() -> Self {
        PaginationConfig {
            items_per_page: 13,
            current_page: 0,
        }
    }
}

/// Identifies a pagination action
#[derive(Debug, Clone, PartialEq)]
pub enum PaginationAction {
    Up,
    Down,
}

/// Identifies a modal action (the only modal is the print ticket one)
#[derive(Debug, Clone, PartialEq)]
pub enum PrintTicketModalActions {
    ShowModal,
    HideModal,
    PrintTicket,
}

/// Holds the state of the print ticker modal
#[derive(Default, Debug, Clone)]
pub struct PrintTicketModalState {
    show_modal: bool,
    //ticket_type: TicketType,
    selected_printer: Option<AlegriaPrinter>, // Printer selected on the selector
    all_printers: Arc<Vec<AlegriaPrinter>>,
    default_printer: Arc<Option<AlegriaPrinter>>,
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
    /// Currently selected table state (helps us identify the currently selected table)
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
    /// Holds the printing modal state
    print_modal: PrintTicketModalState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back

    InitPage, // Intended to be called when first opening the page, asks for the necessary data and executes the appropiate callbacks
    FetchTemporalTickets, // Fetches all the current temporal tickets
    SetTemporalTickets(Vec<TemporalTicket>), // Sets the temporal tickets on the app state
    SetProductCategories(Vec<ProductCategory>), // Sets the product categories on the state
    SetPrinters(Option<AlegriaPrinter>, Vec<AlegriaPrinter>), // Sets the printers on the app state

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

    PrintModalAction(PrintTicketModalActions), // Callback after some action has been requested on the print ticket modal
    UpdateSelectedPrinter(AlegriaPrinter),     // Updates the selected printer
    PrintTicket(Box<SimpleInvoice>), // Callback after creating a simple invoice from the selected temporal ticket in order to print it
    PrintJobCompleted(Result<(), &'static str>), // Callback after print job is completed
}

// Messages/Tasks that need to modify state on the main screen
#[derive(Debug, Clone)]
pub enum BarInstruction {
    Back,
}

#[allow(clippy::derivable_impls)]
impl Default for Bar {
    fn default() -> Self {
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
            product_categories_pagination_state: PaginationConfig::default(),
            product_category_products_pagination_state: PaginationConfig::default(),
            print_modal: PrintTicketModalState::default(),
        }
    }
}

impl Bar {
    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<BarInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            // Asks the parent (app.rs) to go back
            Message::Back => action.add_instruction(BarInstruction::Back),

            // Intended to be called when first opening the page, asks for the necessary data and executes the appropiate callbacks
            Message::InitPage => {
                if let Some(pool) = &self.database {
                    // Get the temporal tickets
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

                    // Get the product categories
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

                action.add_task(Task::perform(AlegriaPrinter::load_printers(), |res| {
                    Message::SetPrinters(res.0, res.1)
                }));
            }
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
            // Sets the product categories on the state
            Message::SetProductCategories(items) => {
                self.currently_selected_product_category = None;
                self.product_category_products = None;
                self.product_categories = items;
            }
            // Sets the printers on the app state
            Message::SetPrinters(default_printer, all_printers) => {
                self.print_modal.selected_printer = default_printer;
                self.print_modal.default_printer =
                    Arc::new(self.print_modal.selected_printer.clone());
                self.print_modal.all_printers = Arc::new(all_printers);
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
                return self.update(Message::FetchTemporalTickets);
            }
            // Callback after we ask to change our current TableLocation
            Message::ChangeCurrentTablesLocation(location) => {
                self.currently_selected_pos_state.location = location;
            }

            // When we click a product on the product list we have to add it to the temporal ticket...
            Message::OnProductClicked(product_id) => {
                // not allow input the current temporal ticket is_some and simple_invoice_id is_some
                let current_ticket = self.temporal_tickets_model.iter().find(|x| {
                    x.table_id == self.currently_selected_pos_state.table_index as i32
                        && x.ticket_location
                            == match_table_location_with_number(
                                self.currently_selected_pos_state.location.clone(),
                            )
                });

                if current_ticket.is_some_and(|x| x.simple_invoice_id.is_some()) {
                    return action;
                }

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
                            simple_invoice_id: None,
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
            Message::OnNumpadKeyClicked(action_type) => {
                // we will need the current ticket to check if there are no more products we will need to delete the temporal ticket
                // and we also need to not allow input the current temporal ticket is_some and simple_invoice_id is_some
                let current_ticket = self.temporal_tickets_model.iter().find(|x| {
                    x.table_id == self.currently_selected_pos_state.table_index as i32
                        && x.ticket_location
                            == match_table_location_with_number(
                                self.currently_selected_pos_state.location.clone(),
                            )
                });

                if current_ticket.is_some_and(|x| x.simple_invoice_id.is_some()) {
                    return action;
                }

                match action_type {
                    // we clicked the delete button of the numpad
                    NumPadAction::Delete => {
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
                                            let value =
                                                &product_quantity[..product_quantity.len() - 1];
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
                }
            }

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
                    // let p_cat_len: i32 =
                    //     self.product_categories.len().try_into().unwrap_or_default();
                    // This aberration happens since adding the printpdf crate which added the deranged crate that causes this,
                    // I think I can either to this or use the line above
                    if next_page_start
                        < <usize as std::convert::TryInto<i32>>::try_into(
                            self.product_categories.len(),
                        )
                        .unwrap_or_default()
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
                    // This aberration happens since adding the printpdf crate which added the deranged crate that causes this
                    if next_page_start
                        < <usize as std::convert::TryInto<i32>>::try_into(
                            self.product_category_products
                                .as_ref()
                                .map(|v| v.len())
                                .unwrap_or(0),
                        )
                        .unwrap_or_default()
                    {
                        self.product_category_products_pagination_state.current_page += 1;
                    }
                }
            },

            // Callback after some action has been requested on the print ticket modal
            Message::PrintModalAction(modal_action) => match modal_action {
                PrintTicketModalActions::ShowModal => {
                    self.print_modal.show_modal = true;
                    action.add_task(widget::focus_next());
                }
                PrintTicketModalActions::HideModal => {
                    self.print_modal.show_modal = false;
                }
                PrintTicketModalActions::PrintTicket => {
                    // we need to get the current ticket in order to print it
                    let current_ticket = self.temporal_tickets_model.iter().find(|x| {
                        x.ticket_location
                            == match_table_location_with_number(
                                self.currently_selected_pos_state.location.clone(),
                            )
                            && x.table_id == self.currently_selected_pos_state.table_index as i32
                    });

                    if let Some(current_ticket) = current_ticket {
                        if let Some(pool) = &self.database {
                            match current_ticket.simple_invoice_id {
                                Some(invoice_id) => {
                                    // if the current ticket is already a simple invoice get it and print it
                                    action.add_task(Task::perform(
                                        SimpleInvoice::get_single(pool.clone(), invoice_id),
                                        |res| match res {
                                            Ok(invoice) => Message::PrintTicket(Box::new(invoice)),
                                            Err(err) => {
                                                eprintln!("{err}");
                                                Message::FetchTemporalTickets
                                            }
                                        },
                                    ));
                                }
                                None => {
                                    // if the current ticket is NOT already a simple invoice create it and print it
                                    action.add_task(Task::perform(
                                        SimpleInvoice::create_from_temporal_ticket(
                                            pool.clone(),
                                            current_ticket.clone(),
                                        ),
                                        |res| match res {
                                            Ok(invoice) => Message::PrintTicket(Box::new(invoice)),
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
            },
            // Updates the selected printer
            Message::UpdateSelectedPrinter(printer) => {
                self.print_modal.selected_printer = Some(printer);
            }
            // Callback after creating a simple invoice from the selected temporal ticket in order to print it
            Message::PrintTicket(invoice) => {
                if let Some(p) = &self.print_modal.selected_printer {
                    let printer = Arc::new(p.clone());
                    action.add_task(Task::perform(
                        printer.print(*invoice),
                        Message::PrintJobCompleted,
                    ));
                }
            }
            // Callback after print job is completed
            Message::PrintJobCompleted(result) => {
                if let Err(e) = result {
                    eprintln!("Error: {}", e);
                }
                self.active_temporal_product = None;
                self.active_temporal_product_field = None;
                return self.update(Message::FetchTemporalTickets);
            }
        }

        action
    }

    const GLOBAL_SPACING: f32 = 6.;
    const GLOBAL_BUTTON_HEIGHT: f32 = 60.;

    /// Returns the view of the bar screen
    pub fn view(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        let header_row = self.view_header_row();

        let bottom_row = row![
            // LEFT SIDE COLUMN
            column![
                // UPPER LEFT SIDE
                row![
                    self.view_tables_grid(),
                    column![self.view_current_ticket_total_price(), self.view_numpad()]
                        .width(235.) //TODO: Maybe this should not be like this but the custom widget also gives some trouble
                        .spacing(spacing)
                ]
                .align_y(Alignment::Center)
                .spacing(spacing),
                // BOTTOM LEFT SIDE
                self.view_current_ticket_products()
            ]
            .spacing(spacing)
            .width(Length::Fill),
            // RIGHT SIDE ROW
            row![
                self.view_product_categories_container(),
                self.view_product_category_products_container(),
            ]
            .spacing(spacing)
            .width(Length::Fill)
        ]
        .spacing(spacing);

        let content = column![header_row, bottom_row]
            .spacing(spacing)
            .height(Length::Fill)
            .width(Length::Fill);

        if self.print_modal.show_modal {
            let print_modal_content = container(self.view_print_modal())
                .width(600)
                .padding(30)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(container::rounded_box);

            modal(
                content,
                print_modal_content,
                Message::PrintModalAction(PrintTicketModalActions::HideModal),
            )
        } else {
            content.into()
        }
    }

    //
    //  VIEW COMPOSING
    //

    const TITLE_TEXT_SIZE: f32 = 25.0;

    /// Returns the view of the header row of the bar screen
    fn view_header_row(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        let back_button = button(
            text(fl!("back"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .on_press(Message::Back)
        .height(button_height);

        let mut header_row = row![
            back_button,
            text(fl!("bar"))
                .size(Self::TITLE_TEXT_SIZE)
                .align_y(Alignment::Center),
            Space::new(Length::Fill, Length::Shrink)
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .spacing(spacing);

        let current_ticket = &self.temporal_tickets_model.iter().find(|x| {
            x.ticket_location
                == match_table_location_with_number(
                    self.currently_selected_pos_state.location.clone(),
                )
                && x.table_id == self.currently_selected_pos_state.table_index as i32
        });

        if let Some(c_ticket) = current_ticket {
            if !c_ticket.products.is_empty() {
                header_row = header_row.push(
                    button(
                        text(fl!("print"))
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center),
                    )
                    .on_press(Message::PrintModalAction(
                        PrintTicketModalActions::ShowModal,
                    ))
                    .height(button_height),
                )
            }
        }

        header_row.into()
    }

    // Controls how many tables there are on a row
    const TABLES_PER_ROW: usize = 5;
    const NUMBER_OF_TABLES: usize = 30;

    /// Returns the view of the tables grid of the application
    fn view_tables_grid(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        let header = Row::new()
            .push(
                button(
                    text(fl!("bar"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ChangeCurrentTablesLocation(TableLocation::Bar))
                .style(|t, s| self.determine_location_button_color(t, s, TableLocation::Bar))
                .height(button_height)
                .width(Length::Fill),
            )
            .push(
                button(
                    text(fl!("restaurant"))
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
                button(
                    text(fl!("garden"))
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

        let mut tables_grid = Column::new().spacing(spacing).width(Length::Fill);
        let mut current_row = Row::new().spacing(spacing).width(Length::Fill);
        for index in 0..Self::NUMBER_OF_TABLES {
            let table_button = button(
                text(format!("{}", index + 1))
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
                current_row = Row::new().spacing(spacing).width(Length::Fill);
            }
        }

        column![header, tables_grid]
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
                button(
                    text(category.name.as_str())
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
        let categories_col = Column::with_children(categories_buttons)
            .spacing(spacing)
            .height(Length::Fill);

        let pagination_buttons = row![
            button(
                text(fl!("up"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::ProductCategoriesPaginationAction(
                PaginationAction::Up,
            ))
            .height(button_height)
            .width(Length::Fill),
            button(
                text(fl!("down"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::ProductCategoriesPaginationAction(
                PaginationAction::Down,
            ))
            .height(button_height)
            .width(Length::Fill)
        ]
        .spacing(spacing)
        .height(Length::Shrink);

        let result_column = column![categories_col, pagination_buttons].height(Length::Fill);

        container(result_column)
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
                        button(
                            text(product.name.as_str())
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
        let products_col = Column::with_children(products_buttons)
            .spacing(spacing)
            .height(Length::Fill);

        let pagination_buttons = row![
            button(
                text(fl!("up"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::ProductCategoryProductsPaginationAction(
                PaginationAction::Up,
            ))
            .height(button_height)
            .width(Length::Fill),
            button(
                text(fl!("down"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::ProductCategoryProductsPaginationAction(
                PaginationAction::Down,
            ))
            .height(button_height)
            .width(Length::Fill)
        ]
        .spacing(spacing)
        .height(Length::Shrink);

        let result_column = column![products_col, pagination_buttons].height(Length::Fill);

        container(result_column)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    /// Returns the view of the product (list) of the currently selected ticket
    fn view_current_ticket_products(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        let current_ticket = &self.temporal_tickets_model.iter().find(|x| {
            x.ticket_location
                == match_table_location_with_number(
                    self.currently_selected_pos_state.location.clone(),
                )
                && x.table_id == self.currently_selected_pos_state.table_index as i32
        });

        if current_ticket.is_some() {
            let mut products_column = Column::new().spacing(spacing);

            for product in &current_ticket.unwrap().products {
                let product_quantity_str = product.quantity.to_string();

                let product_row = Row::new()
                    .push(
                        text(&product.name)
                            .size(25.)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::None),
                    )
                    .push(
                        // Only allow input if the SimpleInvoice has not been yet created
                        if current_ticket.unwrap().simple_invoice_id.is_none() {
                            text_input(&product_quantity_str, &product_quantity_str)
                                .on_focus(move |_| Message::FocusProductQuantity(product.clone()))
                                .on_input(|value| {
                                    Message::TemporalProductInput(product.clone(), value)
                                })
                                .size(25.)
                        } else {
                            text_input(&product_quantity_str, &product_quantity_str).size(25.)
                        },
                    )
                    .push(
                        // Only allow input if the SimpleInvoice has not been yet created
                        if current_ticket.unwrap().simple_invoice_id.is_none() {
                            text_input(&product.price_input, &product.price_input)
                                .on_focus(move |_| Message::FocusProductPrice(product.clone()))
                                .on_input(|value| {
                                    Message::TemporalProductInput(product.clone(), value)
                                })
                                .size(25.)
                        } else {
                            text_input(&product.price_input, &product.price_input).size(25.)
                        },
                    )
                    .spacing(spacing)
                    .align_y(Alignment::Center);

                products_column = products_column.push(product_row);
            }

            Scrollable::new(products_column).into()
        } else {
            row![
                text(fl!("no-products"))
                    .size(25.)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
            ]
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

            text(format!("{:.2}", price)).size(25.).line_height(2.)
        } else {
            text(fl!("unknown")).size(25.).line_height(2.)
        };

        container(text)
            .style(container::bordered_box)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    }

    /// Returns the view of the numpad
    fn view_print_modal(&self) -> Element<Message> {
        if !self.print_modal.all_printers.is_empty() {
            let spacing = Pixels::from(Self::GLOBAL_SPACING);

            let printers_label = text(fl!("printer")).width(Length::Fill);
            let printer_selector = pick_list(
                self.print_modal.all_printers.as_slice(),
                self.print_modal.selected_printer.clone(),
                Message::UpdateSelectedPrinter,
            )
            .width(Length::Fill);

            let submit_button = if self.print_modal.selected_printer.is_some() {
                button(
                    text(fl!("print"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::PrintModalAction(
                    PrintTicketModalActions::PrintTicket,
                ))
                .width(Length::Fill)
            } else {
                button(
                    text(fl!("print"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .width(Length::Fill)
            };

            column![
                column![printers_label, printer_selector].spacing(1.),
                submit_button
            ]
            .spacing(spacing)
            .width(Length::Fill)
            .into()
        } else {
            text("No printers detected...")
                .size(25.)
                .line_height(2.)
                .into()
        }
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
        s: button::Status,
        t_id: usize,
    ) -> button::Style {
        let table_id = t_id as i32;

        // We have it currently selected
        if self.currently_selected_pos_state.table_index as i32 == table_id {
            match s {
                button::Status::Hovered => {
                    return button::primary(t, button::Status::Hovered);
                }
                _ => {
                    return button::primary(t, button::Status::Active);
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
                button::Status::Hovered => {
                    return button::secondary(t, button::Status::Hovered);
                }
                _ => return button::secondary(t, button::Status::Active),
            }

        // there is a pending ticket on this table (we are not currently selecting this ticket)
        } else if current_ticket.is_some_and(|y| {
            match_number_with_temporal_ticket_status(y.ticket_status)
                == TemporalTicketStatus::Pending
        }) {
            match s {
                button::Status::Hovered => {
                    return button::danger(t, button::Status::Hovered);
                }
                _ => return button::danger(t, button::Status::Active),
            }

        // there is a printed ticket on this table (we are not currently selecting this ticket)
        } else if current_ticket.is_some_and(|y| {
            match_number_with_temporal_ticket_status(y.ticket_status)
                == TemporalTicketStatus::Printed
        }) {
            match s {
                button::Status::Hovered => {
                    return button::success(t, button::Status::Hovered);
                }
                _ => return button::success(t, button::Status::Active),
            }
        }

        button::secondary(t, button::Status::Disabled)
    }

    /// Determines the color of the locations buttons using the current location of the state and given which location is which one
    fn determine_location_button_color(
        &self,
        t: &iced::Theme,
        s: button::Status,
        loc: TableLocation,
    ) -> button::Style {
        // we are currently in this location
        if loc == self.currently_selected_pos_state.location {
            match s {
                button::Status::Hovered => button::primary(t, button::Status::Hovered),
                _ => button::primary(t, button::Status::Active),
            }
        } else {
            match s {
                button::Status::Hovered => button::secondary(t, button::Status::Hovered),
                _ => button::secondary(t, button::Status::Active),
            }
        }
    }

    /// Determines the color a button of the tables grid should be given the table index, using the temporal_tickets model
    fn determine_product_category_button_color(
        &self,
        t: &iced::Theme,
        s: button::Status,
        cat_id: Option<i32>,
    ) -> button::Style {
        // we are currently selecting this category
        if self.currently_selected_product_category == cat_id {
            match s {
                button::Status::Hovered => button::primary(t, button::Status::Hovered),
                _ => button::primary(t, button::Status::Active),
            }
        } else {
            match s {
                button::Status::Hovered => button::secondary(t, button::Status::Hovered),
                _ => button::secondary(t, button::Status::Active),
            }
        }
    }

    //
    //  END OF HELPERS
    //
}
