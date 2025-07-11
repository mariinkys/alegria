use std::sync::Arc;

use iced::time::Instant;
use iced::{Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::product::Product;
use crate::alegria::core::models::product_category::ProductCategory;
use crate::alegria::core::models::reservation::Reservation;
use crate::alegria::core::models::simple_invoice::SimpleInvoice;
use crate::alegria::core::models::temporal_product::TemporalProduct;
use crate::alegria::core::models::temporal_ticket::TemporalTicket;
use crate::alegria::core::print::{AlegriaPrinter, TicketType};
use crate::alegria::utils::entities::payment_method::PaymentMethod;
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
    /// Asks to add a toast to the parent state
    AddToast(Toast),
    /// Asks to go back a screen                     
    Back,
    /// Inital Page Loading Completed             
    Loaded(Result<Box<State>, anywho::Error>),

    /// Fetches all the current temporal tickets
    FetchTemporalTickets,
    /// Sets the temporal tickets on the app state
    SetTemporalTickets(Vec<TemporalTicket>),
    /// Fetches the products for a given product category
    FetchProductCategoryProducts(Option<i32>),
    /// Sets the products on the state
    SetProductCategoryProducts(Vec<Product>),
    /// Sets the printers on the app state
    SetPrinters(Box<Option<AlegriaPrinter>>, Vec<AlegriaPrinter>),

    /// Try to go up or down a page on the ProductCategories
    ProductCategoriesPaginationAction(PaginationAction),
    /// Try to go up or down a page on the ProductCategoryProducts
    ProductCategoryProductsPaginationAction(PaginationAction),

    /// Callback after user focus a TemporalProduct
    FocusTemporalProduct(TemporalProduct, TemporalProductField),
    /// text_input of a temporal product
    TemporalProductInput(TemporalProduct, String),

    /// Callback after a numpad number has been clicked
    OnNumpadNumberClicked(u8),
    /// Callback after a numpad key (not a number) has been clicked
    OnNumpadKeyClicked(NumPadAction),

    /// Callback after a table has been clicked
    OnTableChange(usize),
    /// Callback after we ask to change our current TableLocation
    ChangeCurrentTablesLocation(TableLocation),
    /// When we click a product on the product list we have to add it to the temporal ticket...
    OnProductClicked(Option<i32>),

    /// Asks to unlock (delete the related invoice) of a locked ticket
    UnlockTicket(TemporalTicket),
    /// Callback after some action has been requested on the print ticket modal
    PrintModalAction(PrintTicketModalActions),
    /// Updates the selected printer
    UpdateSelectedPrinter(AlegriaPrinter),
    /// Updates the selected ticket type  
    UpdateSelectedTicketType(TicketType),
    /// Callback after creating a simple invoice from the selected temporal ticket in order to print it  
    PrintTicket(Box<SimpleInvoice>),
    /// Callback after print job is completed
    PrintJobCompleted(Result<(), &'static str>),

    /// Attempts to open the pay screen for the given temporal ticket
    OpenPayScreen(TemporalTicket),
    /// Attempts to load the currently occupied reservations for the PayScreeb
    LoadOccupiedReservations,
    /// Callback after loading the currently occupied reservations for the PayScreen
    LoadedOccupiedReservations(Vec<Reservation>),
    /// Updates the currently selected payment method of the pay screen and removes any selected adeudo room id
    UpdateSelectedPaymentMethod(PaymentMethod),
    /// Updates the currently selected adeudo room if needed
    SelectAdeudoSoldRoom(Option<i32>),
    /// Attempts to submit the Pay action of the current pay screen ticket
    PayTicket,
    /// Callback after executing the pay temporal ticket transaction
    PaidTemporalTicket(Result<(), String>),
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
    Pay {
        origin_position: CurrentPosition,
        ticket: TemporalTicket,
        selected_payment_method: PaymentMethod,
        selected_adeudo_room_id: Option<i32>,
        occupied_reservations: Vec<Reservation>,
    },
}

impl std::fmt::Debug for SubScreen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bar { .. } => write!(f, "Bar"),
            Self::Pay { .. } => write!(f, "Pay"),
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
    //default_printer: Arc<Option<AlegriaPrinter>>,
}

/// Identifies a modal action (the only modal is the print ticket one)
#[derive(Debug, Clone)]
pub enum PrintTicketModalActions {
    ShowModal,
    HideModal,
    PrintTicket(TemporalTicket),
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
                Task::perform(init_page(database.clone(), None), Message::Loaded),
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

async fn init_page(
    database: Arc<Pool<Postgres>>,
    position: Option<CurrentPosition>,
) -> Result<Box<State>, anywho::Error> {
    let temporal_tickets = TemporalTicket::get_all(database.clone()).await?;
    let product_categories = ProductCategory::get_all(database.clone()).await?;

    let current_position = position.unwrap_or_default();

    Ok(Box::from(State::Ready {
        sub_screen: SubScreen::Bar {
            temporal_tickets,
            product_categories,
            product_category_products: None,
            pagination: BarPagination::default(),
            current_position,
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
