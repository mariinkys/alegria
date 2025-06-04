use std::sync::Arc;

use iced::time::Instant;
use iced::widget::{container, text};
use iced::{Element, Length, Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::product::Product;
use crate::alegria::core::models::product_category::ProductCategory;
use crate::alegria::core::models::temporal_ticket::TemporalTicket;
use crate::alegria::core::print::{AlegriaPrinter, TicketType};
use crate::alegria::widgets::toast::Toast;

pub struct Bar {
    printer_modal: PrintModal,
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddToast(Toast),                           // Asks to add a toast to the parent state
    Loaded(Result<Box<State>, anywho::Error>), // Inital Page Loading Completed

    FetchTemporalTickets, // Fetches all the current temporal tickets
    SetTemporalTickets(Vec<TemporalTicket>), // Sets the temporal tickets on the app state
    FetchProductCategoryProducts(Option<i32>), // Fetches the products for a given product category
    SetProductCategoryProducts(Vec<Product>), // Sets the products on the state
    SetPrinters(Box<Option<AlegriaPrinter>>, Vec<AlegriaPrinter>), // Sets the printers on the app state
}

#[derive(Debug, Clone)]
pub enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

#[derive(Debug, Clone)]
pub enum SubScreen {
    Bar {
        temporal_tickets: Vec<TemporalTicket>,
        product_categories: Vec<ProductCategory>,
        product_category_products: Option<Vec<Product>>,
    },
    Pay,
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

    pub fn update(
        &mut self,
        message: Message,
        database: &Arc<Pool<Postgres>>,
        _now: Instant,
    ) -> Action {
        match message {
            // Asks to add a toast to the parent state
            Message::AddToast(toast) => Action::AddToast(toast),
            // Inital Page Loading Completed
            Message::Loaded(result) => match result {
                Ok(state) => {
                    self.state = *state;
                    Action::None
                }
                Err(err) => Action::AddToast(Toast::error_toast(err)),
            },
            // Fetches all the current temporal tickets
            Message::FetchTemporalTickets => Action::Run(Task::perform(
                TemporalTicket::get_all(database.clone()),
                |res| match res {
                    Ok(res) => Message::SetTemporalTickets(res),
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err))
                    }
                },
            )),
            // Sets the temporal tickets on the app state
            Message::SetTemporalTickets(res) => {
                #[allow(clippy::collapsible_match)]
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    if let SubScreen::Bar {
                        temporal_tickets, ..
                    } = sub_screen
                    {
                        *temporal_tickets = res;
                    }
                }
                Action::None
            }
            // Fetches all the current temporal tickets
            Message::FetchProductCategoryProducts(category_id) => {
                if let Some(category_id) = category_id {
                    Action::Run(Task::perform(
                        Product::get_all_by_category(database.clone(), category_id),
                        |res| match res {
                            Ok(res) => Message::SetProductCategoryProducts(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::AddToast(Toast::error_toast(err))
                            }
                        },
                    ))
                } else {
                    Action::None
                }
            }
            // Sets the temporal tickets on the app state
            Message::SetProductCategoryProducts(res) => {
                #[allow(clippy::collapsible_match)]
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    if let SubScreen::Bar {
                        product_category_products,
                        ..
                    } = sub_screen
                    {
                        *product_category_products = Some(res);
                    }
                }
                Action::None
            }
            // Sets the printers on the app state
            Message::SetPrinters(default_printer, all_printers) => {
                self.printer_modal = PrintModal {
                    show_modal: false,
                    ticket_type: TicketType::Receipt,
                    selected_printer: default_printer.clone(),
                    all_printers: Arc::new(all_printers),
                    default_printer: Arc::new(*default_printer),
                };
                Action::None
            }
        }
    }

    pub fn view(&self, _now: Instant) -> Element<Message> {
        let content = match &self.state {
            State::Loading => text("Loading..."),
            State::Ready { sub_screen } => match sub_screen {
                SubScreen::Bar {
                    temporal_tickets,
                    product_categories,
                    product_category_products,
                } => text("Data loaded correctly"),
                SubScreen::Pay => todo!(),
            },
        };

        container(content).center(Length::Fill).into()
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
        },
    }))
}
