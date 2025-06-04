use std::sync::Arc;

use iced::Task;
use iced::time::Instant;
use sqlx::{Pool, Postgres};

use super::{Bar, State};
use crate::alegria::{
    core::{
        models::{product::Product, temporal_ticket::TemporalTicket},
        print::TicketType,
    },
    screen::bar::{Action, Message, PaginationAction, PrintModal, SubScreen},
    widgets::toast::Toast,
};

impl Bar {
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
            // Fetches the products for a given product category
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
            // Sets the products on the state
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
            // Try to go up or down a page on the ProductCategories
            Message::ProductCategoriesPaginationAction(action) => {
                #[allow(clippy::collapsible_match)]
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    if let SubScreen::Bar {
                        pagination,
                        product_categories,
                        ..
                    } = sub_screen
                    {
                        match action {
                            PaginationAction::Up => {
                                if pagination.product_categories.current_page > 0 {
                                    pagination.product_categories.current_page -= 1;
                                }
                            }
                            PaginationAction::Down => {
                                let next_page_start = (pagination.product_categories.current_page
                                    + 1)
                                    * pagination.product_categories.items_per_page;
                                // let p_cat_len: i32 =
                                //     self.product_categories.len().try_into().unwrap_or_default();
                                // This aberration happens since adding the printpdf crate which added the deranged crate that causes this,
                                // I think I can either to this or use the line above
                                if next_page_start
                                    < <usize as std::convert::TryInto<i32>>::try_into(
                                        product_categories.len(),
                                    )
                                    .unwrap_or_default()
                                {
                                    pagination.product_categories.current_page += 1;
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            // Try to go up or down a page on the ProductCategoryProducts
            Message::ProductCategoryProductsPaginationAction(action) => {
                #[allow(clippy::collapsible_match)]
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    if let SubScreen::Bar {
                        pagination,
                        product_category_products,
                        ..
                    } = sub_screen
                    {
                        match action {
                            PaginationAction::Up => {
                                if pagination.product_category_products.current_page > 0 {
                                    pagination.product_category_products.current_page -= 1;
                                }
                            }
                            PaginationAction::Down => {
                                let next_page_start =
                                    (pagination.product_category_products.current_page + 1)
                                        * pagination.product_category_products.items_per_page;
                                // This aberration happens since adding the printpdf crate which added the deranged crate that causes this
                                if next_page_start
                                    < <usize as std::convert::TryInto<i32>>::try_into(
                                        product_category_products
                                            .as_ref()
                                            .map(|v| v.len())
                                            .unwrap_or(0),
                                    )
                                    .unwrap_or_default()
                                {
                                    pagination.product_category_products.current_page += 1;
                                }
                            }
                        }
                    }
                }
                Action::None
            }
        }
    }
}
