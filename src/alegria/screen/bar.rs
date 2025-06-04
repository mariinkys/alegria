use std::sync::Arc;

use iced::time::Instant;
use iced::widget::{container, text};
use iced::{Element, Length, Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::product_category::ProductCategory;
use crate::alegria::core::models::temporal_ticket::TemporalTicket;
use crate::alegria::core::print::{AlegriaPrinter, TicketType};
use crate::alegria::widgets::toast::Toast;

pub struct Bar {
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddToast(Toast),                           // Asks to add a toast to the parent state
    Loaded(Result<Box<State>, anywho::Error>), // Inital Page Loading Completed

    FetchTemporalTickets, // Fetches all the current temporal tickets
    SetTemporalTickets(Vec<TemporalTicket>), // Sets the temporal tickets on the app state
    SetProductCategories(Vec<ProductCategory>), // Sets the product categories on the state
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
        printer_modal: PrintModal, // TODO: Printer modal should not be inside the BarScreen since we also need it on the pay screen
    },
    Pay,
}

#[derive(Debug, Clone)]
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
                state: State::Loading,
            },
            Task::perform(init_page(database.clone()), Message::Loaded),
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
            // Sets the product categories on the state
            Message::SetProductCategories(items) => {
                #[allow(clippy::collapsible_match)]
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    if let SubScreen::Bar {
                        product_categories, ..
                    } = sub_screen
                    {
                        *product_categories = items;
                    }
                }
                Action::None
            }
            // Sets the printers on the app state
            Message::SetPrinters(default_printer, all_printers) => {
                #[allow(clippy::collapsible_match)]
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    if let SubScreen::Bar { printer_modal, .. } = sub_screen {
                        *printer_modal = PrintModal {
                            show_modal: false,
                            ticket_type: TicketType::Receipt,
                            selected_printer: default_printer.clone(),
                            all_printers: Arc::new(all_printers),
                            default_printer: Arc::new(*default_printer),
                        };
                    }
                }
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
                    printer_modal,
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
    let (default_printer, all_printers) = AlegriaPrinter::load_printers().await;

    Ok(Box::from(State::Ready {
        sub_screen: SubScreen::Bar {
            temporal_tickets,
            product_categories,
            printer_modal: PrintModal {
                show_modal: false,
                ticket_type: TicketType::Receipt,
                selected_printer: Box::new(default_printer.clone()),
                all_printers: Arc::new(all_printers),
                default_printer: Arc::new(default_printer),
            },
        },
    }))
}
