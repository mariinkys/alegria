use std::sync::Arc;

use iced::Task;
use iced::time::Instant;
use sqlx::{Pool, Postgres};

use super::{Bar, State};
use crate::alegria::{
    core::{
        models::{
            product::Product, reservation::Reservation, simple_invoice::SimpleInvoice,
            temporal_product::TemporalProduct, temporal_ticket::TemporalTicket,
        },
        print::TicketType,
    },
    screen::bar::{
        Action, Message, NumPadAction, PaginationAction, PrintModal, SubScreen,
        TemporalProductField,
    },
    utils::entities::payment_method::PaymentMethod,
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
            Message::Back => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    return match sub_screen {
                        SubScreen::Bar { .. } => Action::Back,
                        SubScreen::Pay {
                            origin_position, ..
                        } => Action::Run(Task::perform(
                            super::init_page(database.clone(), Some(origin_position.to_owned())),
                            Message::Loaded,
                        )),
                    };
                }
                Action::None
            }
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
                        temporal_tickets,
                        active_temporal_product,
                        ..
                    } = sub_screen
                    {
                        *temporal_tickets = res;

                        // we need to update the active_temporal_product to so we can keep updating fields without having to focus again on the field
                        // to update the active_temporal_product, also we want to keep the input of the text field of the currently selected
                        // product, so we don't lose the '.' and we can input decimals
                        if let Some(active_product) = &active_temporal_product.temporal_product {
                            let old_price_input = active_product.price_input.clone();
                            if let Some(product) = temporal_tickets
                                .iter_mut()
                                .flat_map(|ticket| ticket.products.iter_mut())
                                .find(|product| product.id == active_product.id)
                            {
                                if active_temporal_product.temporal_product_field
                                    == Some(TemporalProductField::Price)
                                {
                                    product.price_input = old_price_input;
                                }
                                active_temporal_product.temporal_product = Some(product.clone());
                            }
                        }
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
                    selected_printer: default_printer,
                    all_printers: Arc::new(all_printers),
                    //default_printer: Arc::new(*default_printer),
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
                                if next_page_start
                                    < product_categories.len().try_into().unwrap_or_default()
                                {
                                    pagination.product_categories.current_page += 1;
                                }
                            }
                            PaginationAction::Back => {}
                            PaginationAction::Forward => {}
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

                                if next_page_start
                                    < product_category_products
                                        .as_ref()
                                        .map(|v| v.len())
                                        .unwrap_or(0)
                                        .try_into()
                                        .unwrap_or_default()
                                {
                                    pagination.product_category_products.current_page += 1;
                                }
                            }
                            PaginationAction::Back => {}
                            PaginationAction::Forward => {}
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
            Message::UnlockTicket(ticket) => Action::Run(Task::perform(
                SimpleInvoice::unlock_temporal_ticket(database.clone(), ticket.clone()),
                |res| match res {
                    Ok(_) => Message::FetchTemporalTickets,
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err.to_string()))
                    }
                },
            )),
            Message::PrintModalAction(action) => {
                match action {
                    super::PrintTicketModalActions::ShowModal => {
                        self.printer_modal.show_modal = true;
                        Action::Run(iced::widget::focus_next())
                    }
                    super::PrintTicketModalActions::HideModal => {
                        self.printer_modal.show_modal = false;
                        Action::None
                    }
                    super::PrintTicketModalActions::PrintTicket(ticket) => {
                        match ticket.simple_invoice_id {
                            Some(invoice_id) => {
                                // if the current ticket is already a simple invoice get it and print it
                                Action::Run(Task::perform(
                                    SimpleInvoice::get_single(database.clone(), invoice_id),
                                    |res| match res {
                                        Ok(invoice) => Message::PrintTicket(Box::new(invoice)),
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::FetchTemporalTickets
                                        }
                                    },
                                ))
                            }
                            None => {
                                // if the current ticket is NOT already a simple invoice create it and print it
                                Action::Run(Task::perform(
                                    SimpleInvoice::create_from_temporal_ticket(
                                        database.clone(),
                                        ticket.clone(),
                                    ),
                                    |res| match res {
                                        Ok(invoice) => Message::PrintTicket(Box::new(invoice)),
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::FetchTemporalTickets
                                        }
                                    },
                                ))
                            }
                        }
                    }
                }
            }
            Message::UpdateSelectedPrinter(printer) => {
                self.printer_modal.selected_printer = Box::new(Some(printer));
                Action::None
            }
            Message::UpdateSelectedTicketType(ticket_type) => {
                self.printer_modal.ticket_type = ticket_type;
                Action::None
            }
            // Callback after creating a simple invoice from the selected temporal ticket in order to print it
            Message::PrintTicket(invoice) => {
                // TODO: Change TemporalTicket ticket_status when printed
                if let Some(p) = self.printer_modal.selected_printer.as_ref() {
                    let printer = Arc::new(p.clone());
                    return Action::Run(Task::perform(
                        printer.print(*invoice, self.printer_modal.ticket_type.clone()),
                        Message::PrintJobCompleted,
                    ));
                }
                Action::None
            }
            // Callback after print job is completed
            Message::PrintJobCompleted(result) => {
                if let Err(e) = result {
                    eprintln!("Error: {e}");
                    return Action::AddToast(Toast::error_toast(String::from(e)));
                }

                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Bar {
                        active_temporal_product,
                        ..
                    } = sub_screen
                    {
                        active_temporal_product.temporal_product = None;
                        active_temporal_product.temporal_product_field = None;
                    }
                }

                self.printer_modal.ticket_type = TicketType::default();
                self.update(Message::FetchTemporalTickets, &database.clone(), now)
            }

            Message::OpenPayScreen(ticket) => {
                #[allow(clippy::collapsible_if)]
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    if let SubScreen::Bar {
                        current_position, ..
                    } = &sub_screen
                    {
                        *sub_screen = SubScreen::Pay {
                            origin_position: current_position.clone(),
                            ticket,
                            selected_payment_method: PaymentMethod::Efectivo,
                            occupied_reservations: Vec::new(),
                        };
                        return Action::Run(Task::perform(
                            Reservation::get_occupied(database.clone()),
                            |res| match res {
                                Ok(reservations) => {
                                    Message::LoadedOccupiedReservations(reservations)
                                }
                                Err(err) => {
                                    eprintln!("{err}");
                                    Message::AddToast(Toast::error_toast(err))
                                }
                            },
                        ));
                    }
                }
                Action::None
            }
            Message::LoadedOccupiedReservations(reservations) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Pay {
                        occupied_reservations,
                        ..
                    } = sub_screen
                    {
                        *occupied_reservations = reservations;
                    }
                }
                Action::None
            }
        }
    }
}
