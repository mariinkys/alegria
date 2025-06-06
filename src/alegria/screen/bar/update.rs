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
    screen::bar::{
        Action, Message, NumPadAction, PaginationAction, PrintModal, SubScreen,
        TemporalProductField,
    },
    widgets::toast::Toast,
};

impl Bar {
    #[allow(clippy::only_used_in_recursion)]
    pub fn update(
        &mut self,
        message: Message,
        database: &Arc<Pool<Postgres>>,
        now: Instant,
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
            // Callback after a numpad number has been clicked
            Message::OnNumpadNumberClicked(num) => {
                #[allow(clippy::collapsible_match)]
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Bar {
                        active_temporal_product,
                        ..
                    } = sub_screen
                    {
                        // Extract the data we need before calling self.update()
                        let update_data = if let Some(product) =
                            &active_temporal_product.temporal_product
                        {
                            if let Some(field) = &active_temporal_product.temporal_product_field {
                                match field {
                                    // we add the new number to the corresponding field and pass it as if it was inputed via the keyboard
                                    // to the input handler
                                    TemporalProductField::Quantity => {
                                        let value = format!("{}{}", product.quantity, num);
                                        Some((product.clone(), value))
                                    }
                                    TemporalProductField::Price => {
                                        let value = format!("{}{}", product.price_input, num);
                                        Some((product.clone(), value))
                                    }
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        if let Some((product, value)) = update_data {
                            return self.update(
                                Message::TemporalProductInput(product, value),
                                &database.clone(),
                                now,
                            );
                        }
                    }
                }
                Action::None
            }
            // Callback after a numpad key (not a number) has been clicked
            Message::OnNumpadKeyClicked(action_type) => {
                #[allow(clippy::collapsible_match)]
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Bar {
                        active_temporal_product,
                        temporal_tickets,
                        current_position,
                        ..
                    } = sub_screen
                    {
                        // we will need the current ticket to check if there are no more products we will need to delete the temporal ticket
                        // and we also need to not allow input the current temporal ticket is_some and simple_invoice_id is_some
                        let current_ticket = temporal_tickets.iter().find(|x| {
                            x.ticket_location
                                == super::match_table_location_with_number(
                                    &current_position.table_location,
                                )
                                && x.table_id == current_position.table_index
                        });

                        if current_ticket.is_some_and(|x| x.simple_invoice_id.is_some()) {
                            return Action::None;
                        }

                        match action_type {
                            // we clicked the delete button of the numpad
                            NumPadAction::Delete => {
                                // if we have a product selected we delete that product
                                if let Some(active_product) =
                                    &active_temporal_product.temporal_product
                                {
                                    let mut tasks = vec![];
                                    tasks.push(Task::perform(
                                        TemporalProduct::delete(
                                            database.clone(),
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

                                    active_temporal_product.temporal_product = None;
                                    active_temporal_product.temporal_product_field = None;

                                    // check if there are no more products we will need to delete the temporal ticket
                                    if let Some(ticket) = current_ticket
                                        && ticket.products.len() == 1
                                    {
                                        tasks.push(Task::perform(
                                            TemporalTicket::delete(
                                                database.clone(),
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

                                    return Action::Run(Task::batch(tasks));

                                // if we don't have a product selected but there is a ticket and we pressed delete
                                } else if let Some(ticket) = current_ticket {
                                    if let Some(product) = ticket.products.first() {
                                        let mut tasks = vec![];
                                        // we delete the first product of the list
                                        tasks.push(Task::perform(
                                            TemporalProduct::delete(
                                                database.clone(),
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
                                            tasks.push(Task::perform(
                                                TemporalTicket::delete(
                                                    database.clone(),
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
                                        return Action::Run(Task::batch(tasks));
                                    }
                                    return Action::None;
                                }
                            }
                            // we clicked the erase button of the numpad
                            NumPadAction::Erase => {
                                // Extract the product and field data before dropping the mutable borrow
                                let update_data = if let Some(product) =
                                    &active_temporal_product.temporal_product
                                {
                                    if let Some(field) =
                                        &active_temporal_product.temporal_product_field
                                    {
                                        match field {
                                            TemporalProductField::Quantity => {
                                                let product_quantity = product.quantity.to_string();
                                                if product_quantity.len() > 1 {
                                                    let value = &product_quantity
                                                        [..product_quantity.len() - 1];
                                                    Some((product.clone(), value.to_string()))
                                                } else {
                                                    // if we only have one "char" we put a 0
                                                    Some((product.clone(), 0.to_string()))
                                                }
                                            }
                                            TemporalProductField::Price => {
                                                let product_price = &product.price_input;
                                                if product_price.len() > 1 {
                                                    let value =
                                                        &product_price[..product_price.len() - 1];
                                                    Some((product.clone(), value.to_string()))
                                                } else {
                                                    Some((product.clone(), String::new()))
                                                }
                                            }
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                // Now call self.update with the extracted data
                                if let Some((product, value)) = update_data {
                                    return self.update(
                                        Message::TemporalProductInput(product, value),
                                        &database.clone(),
                                        now,
                                    );
                                }
                            }
                            // we clicked the '.' button of the numpad
                            NumPadAction::Decimal => {
                                // Extract the product data before dropping the mutable borrow
                                let update_data = if let Some(product) =
                                    &active_temporal_product.temporal_product
                                {
                                    if let Some(field) =
                                        &active_temporal_product.temporal_product_field
                                    {
                                        // only the price can be decimal
                                        if *field == TemporalProductField::Price {
                                            Some((
                                                product.clone(),
                                                format!("{}.", product.price_input),
                                            ))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                // Now call self.update with the extracted data
                                if let Some((product, value)) = update_data {
                                    return self.update(
                                        Message::TemporalProductInput(product, value),
                                        &database.clone(),
                                        now,
                                    );
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            // Callback after a table has been clicked
            Message::OnTableChange(table_index) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Bar {
                        active_temporal_product,
                        current_position,
                        ..
                    } = sub_screen
                    {
                        current_position.table_index = table_index as i32;
                        active_temporal_product.temporal_product = None;
                        active_temporal_product.temporal_product_field = None;
                        return self.update(Message::FetchTemporalTickets, &database.clone(), now);
                    }
                }
                Action::None
            }
            // Callback after we ask to change our current TableLocation
            Message::ChangeCurrentTablesLocation(location) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Bar {
                        current_position, ..
                    } = sub_screen
                    {
                        current_position.table_location = location;
                    }
                }
                Action::None
            }

            // When we click a product on the product list we have to add it to the temporal ticket...
            Message::OnProductClicked(product_id) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Bar {
                        temporal_tickets,
                        current_position,
                        active_temporal_product,
                        ..
                    } = sub_screen
                    {
                        // not allow input the current temporal ticket is_some and simple_invoice_id is_some
                        let current_ticket = temporal_tickets.iter().find(|x| {
                            x.ticket_location
                                == super::match_table_location_with_number(
                                    &current_position.table_location,
                                )
                                && x.table_id == current_position.table_index
                        });

                        if current_ticket.is_some_and(|x| x.simple_invoice_id.is_some()) {
                            return Action::None;
                        }

                        if let Some(new_product_id) = product_id {
                            // Deselect the active temporal product
                            active_temporal_product.temporal_product = None;

                            let temporal_ticket = TemporalTicket {
                                id: None,
                                table_id: current_position.table_index,
                                ticket_location: super::match_table_location_with_number(
                                    &current_position.table_location.clone(),
                                ),
                                ticket_status: 0,
                                simple_invoice_id: None,
                                products: Vec::new(),
                            };

                            // Upsert a temporal ticket with the clicked product
                            return Action::Run(Task::perform(
                                TemporalTicket::upsert_ticket_by_id_and_tableloc(
                                    database.clone(),
                                    temporal_ticket,
                                    new_product_id,
                                ),
                                |res| match res {
                                    Ok(_) => Message::FetchTemporalTickets,
                                    Err(err) => {
                                        eprintln!("{err}");
                                        Message::AddToast(Toast::error_toast(err.to_string()))
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
