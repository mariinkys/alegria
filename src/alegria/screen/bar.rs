use std::sync::Arc;

use iced::time::Instant;
use iced::{Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::product::Product;
use crate::alegria::core::models::product_category::ProductCategory;
use crate::alegria::core::models::temporal_product::TemporalProduct;
use crate::alegria::core::models::temporal_ticket::TemporalTicket;
use crate::alegria::core::print::{AlegriaPrinter, TicketType};
use crate::alegria::utils::pagination::{PaginationAction, PaginationConfig};
use crate::alegria::widgets::toast::Toast;

mod update;
mod view;

pub struct Bar {
    printer_modal: PrintModal,
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddToast(Toast),                           // Asks to add a toast to the parent state
    Back,                                      // Asks to go back a screen
    Loaded(Result<Box<State>, anywho::Error>), // Inital Page Loading Completed

    FetchTemporalTickets, // Fetches all the current temporal tickets
    SetTemporalTickets(Vec<TemporalTicket>), // Sets the temporal tickets on the app state
    FetchProductCategoryProducts(Option<i32>), // Fetches the products for a given product category
    SetProductCategoryProducts(Vec<Product>), // Sets the products on the state
    SetPrinters(Box<Option<AlegriaPrinter>>, Vec<AlegriaPrinter>), // Sets the printers on the app state

    ProductCategoriesPaginationAction(PaginationAction), // Try to go up or down a page on the ProductCategories
    ProductCategoryProductsPaginationAction(PaginationAction), // Try to go up or down a page on the ProductCategoryProducts

    FocusTemporalProduct(TemporalProduct, TemporalProductField), // Callback after user focus a TemporalProduct
    TemporalProductInput(TemporalProduct, String),               // text_input of a temporal product

    OnNumpadNumberClicked(u8), // Callback after a numpad number has been clicked
    OnNumpadKeyClicked(NumPadAction), // Callback after a numpad key (not a number) has been clicked

    OnTableChange(usize), // Callback after a table has been clicked
    ChangeCurrentTablesLocation(TableLocation), // Callback after we ask to change our current TableLocation
    OnProductClicked(Option<i32>), // When we click a product on the product list we have to add it to the temporal ticket...

    UnlockTicket(TemporalTicket), // Asks to unlock (delete the related invoice) of a locked ticket
}

// We only need to derive Debug and Clone because we're passing a State through the Loaded Message, there may be a better way to do this
// that makes us able to remove this two Derives, for now switching to a manual implementation of Debug helps us not lose
// speed because of the derives (same on SubScreen enum)
#[derive(Clone)]
pub enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loading => write!(f, "Loading"),
            Self::Ready { .. } => write!(f, "Ready"),
        }
    }
}

#[derive(Clone)]
pub enum SubScreen {
    Bar {
        temporal_tickets: Vec<TemporalTicket>,
        product_categories: Vec<ProductCategory>,
        product_category_products: Option<Vec<Product>>,
        pagination: BarPagination,
        current_position: CurrentPosition,
        active_temporal_product: ActiveTemporalProduct,
    },
    Pay,
}

impl std::fmt::Debug for SubScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bar { .. } => write!(f, "Bar"),
            Self::Pay => write!(f, "Pay"),
        }
    }
}

/// Holds the state of the pagination for various entities of the BarScreen
#[derive(Default, Debug, Clone)]
pub struct BarPagination {
    product_categories: PaginationConfig,
    product_category_products: PaginationConfig,
}

/// Holds the pagination state (generic, for various entities)
#[derive(Default, Debug, Clone)]
pub struct CurrentPosition {
    /// Currently selected table location
    table_location: TableLocation,
    /// Currently selected table index
    table_index: i32,
    /// Currently selected product_category id (needed for correct button styling)
    selected_product_category: Option<i32>,
}

#[derive(Default, Debug, Clone)]
pub struct ActiveTemporalProduct {
    /// Keeps track of which temporal product is active (within a temporal ticket) in order to be able to modify it with the NumPad
    temporal_product: Option<TemporalProduct>,
    /// Keeps track of which temporal product field is active (within a temporal product) in order to be able to modify it with the NumPad
    temporal_product_field: Option<TemporalProductField>,
}

/// What field of a TemporalProduct are we currently focusing on?
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalProductField {
    Quantity,
    Price,
}

/// Defines the different locations in which a table can be located at
#[derive(Default, Debug, Clone, PartialEq)]
pub enum TableLocation {
    #[default]
    Bar,
    Resturant,
    Garden,
}

/// Identifies an action of the numpad
#[derive(Debug, Clone, PartialEq)]
pub enum NumPadAction {
    Delete,
    Erase,
    Decimal,
}

#[derive(Default, Debug, Clone)]
pub struct PrintModal {
    show_modal: bool,
    ticket_type: TicketType,
    selected_printer: Box<Option<AlegriaPrinter>>,
    all_printers: Arc<Vec<AlegriaPrinter>>,
    default_printer: Arc<Option<AlegriaPrinter>>,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl Bar {
    pub fn new(database: &Arc<Pool<Postgres>>) -> (Self, Task<Message>) {
        (
            Self {
                printer_modal: PrintModal::default(),
                state: State::Loading,
            },
            Task::batch([
                Task::perform(init_page(database.clone()), Message::Loaded),
                Task::perform(AlegriaPrinter::load_printers(), |res| {
                    Message::SetPrinters(Box::from(res.0), res.1)
                }),
            ]),
        )
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}

async fn init_page(database: Arc<Pool<Postgres>>) -> Result<Box<State>, anywho::Error> {
    let temporal_tickets = TemporalTicket::get_all(database.clone()).await?;
    let product_categories = ProductCategory::get_all(database.clone()).await?;

    Ok(Box::from(State::Ready {
        sub_screen: SubScreen::Bar {
            temporal_tickets,
            product_categories,
            product_category_products: None,
            pagination: BarPagination::default(),
            current_position: CurrentPosition::default(),
            active_temporal_product: ActiveTemporalProduct::default(),
        },
    }))
}

pub fn match_table_location_with_number(tl: &TableLocation) -> i32 {
    match tl {
        TableLocation::Bar => 0,
        TableLocation::Resturant => 1,
        TableLocation::Garden => 2,
    }
}
