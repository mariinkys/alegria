use std::sync::Arc;

use iced::time::Instant;
use iced::widget::text;
use iced::{Element, Subscription, Task};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::payment_method::PaymentMethod;
use crate::alegria::core::models::product_category::ProductCategory;
use crate::alegria::core::models::temporal_ticket::TemporalTicket;
use crate::alegria::core::print::{AlegriaPrinter, TicketType};
use crate::alegria::widgets::toast::Toast;

pub struct Bar {
    state: State,
}

#[derive(Debug, Clone)]
pub enum Message {
    None,
    AddToast(Toast),

    FetchTemporalTickets, // Fetches all the current temporal tickets
    SetTemporalTickets(Vec<TemporalTicket>), // Sets the temporal tickets on the app state
    SetProductCategories(Vec<ProductCategory>), // Sets the product categories on the state
    SetPrinters(Box<Option<AlegriaPrinter>>, Vec<AlegriaPrinter>), // Sets the printers on the app state
}

pub enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

pub enum SubScreen {
    Bar {
        temporal_tickets: Vec<TemporalTicket>,
        product_categories: Vec<ProductCategory>,
        printer_modal: PrintModal,
    },
    Pay,
}

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
        let mut init_tasks = vec![];

        // Get the temporal tickets
        init_tasks.push(Task::perform(
            TemporalTicket::get_all(database.clone()),
            |res| match res {
                Ok(res) => Message::SetTemporalTickets(res),
                Err(err) => {
                    eprintln!("{err}");
                    Message::AddToast(Toast::error_toast(err))
                }
            },
        ));

        // Get the product categories
        init_tasks.push(Task::perform(
            ProductCategory::get_all(database.clone()),
            |res| match res {
                Ok(res) => Message::SetProductCategories(res),
                Err(err) => {
                    eprintln!("{err}");
                    Message::AddToast(Toast::error_toast(err))
                }
            },
        ));

        // Get the printers
        init_tasks.push(Task::perform(AlegriaPrinter::load_printers(), |res| {
            Message::SetPrinters(Box::new(res.0), res.1)
        }));

        (
            Self {
                state: State::Loading,
            },
            Task::batch(init_tasks),
        )
    }

    pub fn update(
        &mut self,
        message: Message,
        database: &Arc<Pool<Postgres>>,
        now: Instant,
    ) -> Action {
        match message {
            Message::None => Action::None,
            Message::AddToast(toast) => Action::AddToast(toast),
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

    pub fn view(&self, now: Instant) -> Element<Message> {
        text("text").into()
    }

    pub fn subscription(&self, now: Instant) -> Subscription<Message> {
        Subscription::none()
    }
}
