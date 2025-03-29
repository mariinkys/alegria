// SPDX-License-Identifier: GPL-3.0-only

/// The main issue or dificulty with this page for now is the fact that everytime the user does something
/// we need to save it on the database and refresh, that way the data persists even if there is a power
/// shortage...
mod update;
mod view;

use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    alegria::{
        core::{
            models::{
                payment_method::PaymentMethod, product::Product, product_category::ProductCategory,
                simple_invoice::SimpleInvoice, temporal_product::TemporalProduct,
                temporal_ticket::TemporalTicket,
            },
            print::AlegriaPrinter,
        },
        widgets::toast::Toast,
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

#[derive(Default, Debug, Clone, PartialEq)]
pub enum TicketType {
    Invoice,
    #[default]
    Receipt,
}

impl std::fmt::Display for TicketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketType::Invoice => write!(f, "{}", fl!("invoice")),
            TicketType::Receipt => write!(f, "{}", fl!("receipt")),
        }
    }
}

/// Holds the state of the print ticker modal
#[derive(Default, Debug, Clone)]
pub struct PrintTicketModalState {
    show_modal: bool,
    ticket_type: TicketType, // The kind of ticket we want to print
    selected_printer: Option<AlegriaPrinter>, // Printer selected on the selector
    all_printers: Arc<Vec<AlegriaPrinter>>,
    default_printer: Arc<Option<AlegriaPrinter>>,
}

/// Represents a page of the bar screen
#[derive(PartialEq)]
pub enum BarScreen {
    Home,
    Pay,
}

/// Holds the state of the pay screen inputs...
#[derive(Default, Debug, Clone)]
pub struct PayScreenState {
    payment_methods: Vec<PaymentMethod>,
    selected_payment_method: Option<PaymentMethod>,
}

pub struct Bar {
    /// Database of the application
    pub database: Option<Arc<PgPool>>,
    /// Page Toasts
    toasts: Vec<Toast>,
    /// Determines which is the current bar screen
    bar_screen: BarScreen,
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
    /// Holds the satate of the payscreen inputs...
    pay_screen: PayScreenState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Back, // Asks the parent (app.rs) to go back

    AddToast(Toast),   // Adds the given toast to the state to be shown on screen
    CloseToast(usize), // Callback after clicking the close toast button

    InitPage, // Intended to be called when first opening the page, asks for the necessary data and executes the appropiate callbacks
    FetchTemporalTickets, // Fetches all the current temporal tickets
    SetTemporalTickets(Vec<TemporalTicket>), // Sets the temporal tickets on the app state
    SetProductCategories(Vec<ProductCategory>), // Sets the product categories on the state
    SetPrinters(Option<AlegriaPrinter>, Vec<AlegriaPrinter>), // Sets the printers on the app state
    SetPaymentMethods(Vec<PaymentMethod>), // Sets the payment methods on the app state

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
    UpdateSelectedTicketType(TicketType),      // Updates the selected ticket type
    PrintTicket(Box<SimpleInvoice>), // Callback after creating a simple invoice from the selected temporal ticket in order to print it
    PrintJobCompleted(Result<(), &'static str>), // Callback after print job is completed
    UnlockTicket(TemporalTicket), // Asks to unlock (delete the related invoice) of a locked ticket

    OpenPayScreen, // Tries to open the pay screen for the currently selected TemporalTicket
    ChangeSelectedPaymentMethod(PaymentMethod), // Changes the currently selected payment method for the given one
    PayTemporalTicket(i32), // Tries to execute the pay transaction for the given TemporalTicketId
    PaidTemporalTicket(Result<(), String>), // Callback after executing the pay temporal ticket transaction
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
            toasts: Vec::new(),
            bar_screen: BarScreen::Home,
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
            pay_screen: PayScreenState::default(),
        }
    }
}
