use std::sync::Arc;

use iced::Task;
use iced::time::Instant;
use sqlx::{Pool, Postgres};

use super::{Bar, State};
use crate::alegria::{
    core::{
        models::{
            product::Product, temporal_product::TemporalProduct, temporal_ticket::TemporalTicket,
        },
        print::TicketType,
    },
    screen::bar::{Action, Message, PaginationAction, PrintModal, SubScreen, TemporalProductField},
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
            Message::AddToast(toast) => Action::AddToast(toast),
            Message::Back => Action::Back,
            Message::Loaded(result) => match result {
                Ok(state) => {
                    self.state = *state;
                    Action::None
                }
                Err(err) => Action::AddToast(Toast::error_toast(err)),
            },
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
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Bar {
                        temporal_tickets, ..
                    } = sub_screen
                    {
                        *temporal_tickets = res;
                    }
                }
                Action::None
            }
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
            Message::SetProductCategoryProducts(res) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
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
            Message::ProductCategoriesPaginationAction(action) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
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
            Message::ProductCategoryProductsPaginationAction(action) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
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
            Message::FocusTemporalProduct(temporal_product, field) => {
                #[allow(clippy::collapsible_match)]
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Bar {
                        active_temporal_product,
                        ..
                    } = sub_screen
                    {
                        active_temporal_product.temporal_product = Some(temporal_product);
                        active_temporal_product.temporal_product_field = Some(field);
                    }
                }
                Action::None
            }
            Message::TemporalProductInput(temporal_product, new_value) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Bar {
                        active_temporal_product,
                        ..
                    } = sub_screen
                    {
                        #[allow(clippy::collapsible_match)]
                        if let Some(field) = &active_temporal_product.temporal_product_field {
                            let mut mutable_product = temporal_product;

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
                                    let ignore_action = new_value.len()
                                        > mutable_product.price_input.len()
                                        && mutable_product.price_input.find('.').is_some_and(
                                            |idx| mutable_product.price_input.len() - idx > 2,
                                        );

                                    if !ignore_action {
                                        if let Ok(num) = new_value.parse::<f32>() {
                                            mutable_product.price = Some(num);

                                            if let Some(active_product) =
                                                &mut active_temporal_product.temporal_product
                                            {
                                                active_product.price_input = new_value;
                                            }
                                        } else if new_value.is_empty() {
                                            mutable_product.price = Some(0.0);

                                            if let Some(active_product) =
                                                &mut active_temporal_product.temporal_product
                                            {
                                                active_product.price_input = new_value;
                                            }
                                        }
                                    }
                                }
                            }

                            return Action::Run(Task::perform(
                                TemporalProduct::edit(database.clone(), mutable_product),
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
                Action::None
            }
        }
    }
}
