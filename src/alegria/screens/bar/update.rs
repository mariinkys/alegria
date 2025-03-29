use std::sync::Arc;

use iced::{Task, widget};

use crate::alegria::{
    action::AlegriaAction,
    core::{
        models::{
            payment_method::PaymentMethod, product::Product, product_category::ProductCategory,
            simple_invoice::SimpleInvoice, temporal_product::TemporalProduct,
            temporal_ticket::TemporalTicket,
        },
        print::AlegriaPrinter,
    },
    utils::{error_toast, match_table_location_with_number},
};

use super::{
    Bar, BarInstruction, BarScreen, Message, NumPadAction, PaginationAction,
    PrintTicketModalActions, TemporalProductField, TicketType,
};

impl Bar {
    /// Handles messages emitted by the application and its widgets.
    pub fn update(&mut self, message: Message) -> AlegriaAction<BarInstruction, Message> {
        let mut action = AlegriaAction::new();

        match message {
            // Asks the parent (app.rs) to go back
            Message::Back => match self.bar_screen {
                BarScreen::Home => action.add_instruction(BarInstruction::Back),
                BarScreen::Pay => {
                    self.bar_screen = BarScreen::Home;
                }
            },

            // Adds the given toast to the state to be shown on screen
            Message::AddToast(toast) => {
                self.toasts.push(toast);
            }
            // Callback after clicking the close toast button
            Message::CloseToast(index) => {
                self.toasts.remove(index);
            }

            // Intended to be called when first opening the page, asks for the necessary data and executes the appropiate callbacks
            Message::InitPage => {
                if let Some(pool) = &self.database {
                    // Get the temporal tickets
                    action.add_task(Task::perform(
                        TemporalTicket::get_all(pool.clone()),
                        |res| match res {
                            Ok(res) => Message::SetTemporalTickets(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::AddToast(error_toast(err.to_string()))
                            }
                        },
                    ));

                    // Get the product categories
                    action.add_task(Task::perform(
                        ProductCategory::get_all(pool.clone()),
                        |res| match res {
                            Ok(items) => Message::SetProductCategories(items),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::AddToast(error_toast(err.to_string()))
                            }
                        },
                    ));

                    // Get the payment methods
                    action.add_task(Task::perform(PaymentMethod::get_all(pool.clone()), |res| {
                        match res {
                            Ok(items) => Message::SetPaymentMethods(items),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::AddToast(error_toast(err.to_string()))
                            }
                        }
                    }));
                }

                action.add_task(Task::perform(AlegriaPrinter::load_printers(), |res| {
                    Message::SetPrinters(res.0, res.1)
                }));
            }
            // Fetches all the current temporal tickets
            Message::FetchTemporalTickets => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        TemporalTicket::get_all(pool.clone()),
                        |res| match res {
                            Ok(res) => Message::SetTemporalTickets(res),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::AddToast(error_toast(err.to_string()))
                            }
                        },
                    ));
                }
            }
            // Sets the temporal tickets on the app state
            Message::SetTemporalTickets(res) => {
                self.temporal_tickets_model = res;

                // we need to update the active_temporal_product to so we can keep updating fields without having to focus again on the field
                // to update the active_temporal_product, also we want to keep the input of the text field of the currently selected
                // product, so we don't lose the '.' and we can input decimals
                if let Some(active_product) = &self.active_temporal_product {
                    let old_price_input = active_product.price_input.clone();
                    if let Some(product) = self
                        .temporal_tickets_model
                        .iter_mut()
                        .flat_map(|ticket| ticket.products.iter_mut())
                        .find(|product| product.id == active_product.id)
                    {
                        if self.active_temporal_product_field == Some(TemporalProductField::Price) {
                            product.price_input = old_price_input;
                        }
                        self.active_temporal_product = Some(product.clone());
                    }
                }
            }
            // Sets the product categories on the state
            Message::SetProductCategories(items) => {
                self.currently_selected_product_category = None;
                self.product_category_products = None;
                self.product_categories = items;
            }
            // Sets the printers on the app state
            Message::SetPrinters(default_printer, all_printers) => {
                self.print_modal.selected_printer = default_printer;
                self.print_modal.default_printer =
                    Arc::new(self.print_modal.selected_printer.clone());
                self.print_modal.all_printers = Arc::new(all_printers);
            }
            // Sets the payment methods on the app state
            Message::SetPaymentMethods(p_methods) => {
                if !p_methods.is_empty() {
                    self.pay_screen.selected_payment_method = p_methods.first().cloned();
                }
                self.pay_screen.payment_methods = p_methods;
            }

            // Fetches the products for a given product category
            Message::FetchProductCategoryProducts(product_category_id) => {
                if let Some(pool) = &self.database {
                    self.currently_selected_product_category = product_category_id;
                    action.add_task(Task::perform(
                        Product::get_all_by_category(
                            pool.clone(),
                            product_category_id.unwrap_or_default(),
                        ),
                        |res| match res {
                            Ok(items) => Message::SetProductCategoryProducts(Some(items)),
                            Err(err) => {
                                eprintln!("{err}");
                                Message::AddToast(error_toast(err.to_string()))
                            }
                        },
                    ));
                }
            }
            // Sets the products on the state
            Message::SetProductCategoryProducts(items) => {
                self.product_category_products = items;
            }

            // Callback after a table has been clicked
            Message::OnTableChange(table_index) => {
                self.currently_selected_pos_state.table_index = table_index;
                self.active_temporal_product = None;
                self.active_temporal_product_field = None;
                return self.update(Message::FetchTemporalTickets);
            }
            // Callback after we ask to change our current TableLocation
            Message::ChangeCurrentTablesLocation(location) => {
                self.currently_selected_pos_state.location = location;
            }

            // When we click a product on the product list we have to add it to the temporal ticket...
            Message::OnProductClicked(product_id) => {
                // not allow input the current temporal ticket is_some and simple_invoice_id is_some
                let current_ticket = self.temporal_tickets_model.iter().find(|x| {
                    x.table_id == self.currently_selected_pos_state.table_index as i32
                        && x.ticket_location
                            == match_table_location_with_number(
                                self.currently_selected_pos_state.location.clone(),
                            )
                });

                if current_ticket.is_some_and(|x| x.simple_invoice_id.is_some()) {
                    return action;
                }

                if let Some(new_product_id) = product_id {
                    if let Some(pool) = &self.database {
                        // Deselect the active temporal product
                        self.active_temporal_product = None;

                        let temporal_ticket = TemporalTicket {
                            id: None,
                            table_id: self.currently_selected_pos_state.table_index as i32,
                            ticket_location: match_table_location_with_number(
                                self.currently_selected_pos_state.location.clone(),
                            ),
                            ticket_status: 0,
                            simple_invoice_id: None,
                            products: Vec::new(),
                        };

                        // Upsert a temporal ticket with the clicked product
                        action.add_task(Task::perform(
                            TemporalTicket::upsert_ticket_by_id_and_tableloc(
                                pool.clone(),
                                temporal_ticket,
                                new_product_id,
                            ),
                            |res| match res {
                                Ok(_) => Message::FetchTemporalTickets,
                                Err(err) => {
                                    eprintln!("{err}");
                                    Message::AddToast(error_toast(err.to_string()))
                                }
                            },
                        ));
                    }
                }
            }

            // Callback after a numpad number has been clicked
            Message::OnNumpadNumberClicked(num) => {
                if let Some(product) = &self.active_temporal_product {
                    if let Some(field) = &self.active_temporal_product_field {
                        match field {
                            // we add the new number to the corresponding field and pass it as if it was inputed via the keyboard
                            // to the input handler
                            TemporalProductField::Quantity => {
                                let value = format!("{}{}", product.quantity, num);
                                return self
                                    .update(Message::TemporalProductInput(product.clone(), value));
                            }
                            TemporalProductField::Price => {
                                let value = format!("{}{}", product.price_input, num);
                                return self
                                    .update(Message::TemporalProductInput(product.clone(), value));
                            }
                        }
                    }
                }
            }
            // Callback after a numpad key (not a number) has been clicked
            Message::OnNumpadKeyClicked(action_type) => {
                // we will need the current ticket to check if there are no more products we will need to delete the temporal ticket
                // and we also need to not allow input the current temporal ticket is_some and simple_invoice_id is_some
                let current_ticket = self.temporal_tickets_model.iter().find(|x| {
                    x.table_id == self.currently_selected_pos_state.table_index as i32
                        && x.ticket_location
                            == match_table_location_with_number(
                                self.currently_selected_pos_state.location.clone(),
                            )
                });

                if current_ticket.is_some_and(|x| x.simple_invoice_id.is_some()) {
                    return action;
                }

                match action_type {
                    // we clicked the delete button of the numpad
                    NumPadAction::Delete => {
                        // if we have a product selected we delete that product
                        if let Some(active_product) = &self.active_temporal_product {
                            if let Some(pool) = &self.database {
                                action.add_task(Task::perform(
                                    TemporalProduct::delete(
                                        pool.clone(),
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

                                self.active_temporal_product = None;
                                self.active_temporal_product_field = None;

                                // check if there are no more products we will need to delete the temporal ticket
                                if let Some(ticket) = current_ticket {
                                    if ticket.products.len() == 1 {
                                        action.add_task(Task::perform(
                                            TemporalTicket::delete(
                                                pool.clone(),
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
                                }
                            }
                        // if we don't have a product selected but there is a ticket and we pressed delete
                        } else if let Some(ticket) = current_ticket {
                            if let Some(product) = ticket.products.first() {
                                if let Some(pool) = &self.database {
                                    // we delete the first product of the list
                                    action.add_task(Task::perform(
                                        TemporalProduct::delete(
                                            pool.clone(),
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
                                        action.add_task(Task::perform(
                                            TemporalTicket::delete(
                                                pool.clone(),
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
                                }
                            }
                        }
                    }
                    // we clicked the erase button of the numpad
                    NumPadAction::Erase => {
                        if let Some(product) = &self.active_temporal_product {
                            if let Some(field) = &self.active_temporal_product_field {
                                match field {
                                    // we substract a char of the corresponding field and pass it to the
                                    // input update function as if it was inputed via keyboard
                                    TemporalProductField::Quantity => {
                                        let product_quantity = product.quantity.to_string();
                                        if product_quantity.len() > 1 {
                                            let value =
                                                &product_quantity[..product_quantity.len() - 1];
                                            return self.update(Message::TemporalProductInput(
                                                product.clone(),
                                                value.to_string(),
                                            ));
                                        } else {
                                            // if we only have one "char" we put a 0
                                            return self.update(Message::TemporalProductInput(
                                                product.clone(),
                                                0.to_string(),
                                            ));
                                        }
                                    }
                                    TemporalProductField::Price => {
                                        let product_price = &product.price_input;
                                        if product_price.len() > 1 {
                                            let value = &product_price[..product_price.len() - 1];

                                            return self.update(Message::TemporalProductInput(
                                                product.clone(),
                                                value.to_string(),
                                            ));
                                        } else {
                                            return self.update(Message::TemporalProductInput(
                                                product.clone(),
                                                String::new(),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // we clicked the '.' button of the numpad
                    NumPadAction::Decimal => {
                        if let Some(product) = &self.active_temporal_product {
                            if let Some(field) = &self.active_temporal_product_field {
                                // only the price can be decimal
                                if *field == TemporalProductField::Price {
                                    return self.update(Message::TemporalProductInput(
                                        product.clone(),
                                        format!("{}.", product.price_input),
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            // Callback after user focus the quantity field of a TemporalProduct
            Message::FocusProductQuantity(product) => {
                self.active_temporal_product = Some(product);
                self.active_temporal_product_field = Some(TemporalProductField::Quantity);
            }
            // Callback after user focus the price field of a TemporalProduct
            Message::FocusProductPrice(product) => {
                self.active_temporal_product = Some(product);
                self.active_temporal_product_field = Some(TemporalProductField::Price);
            }
            // text_input of a temporal product
            Message::TemporalProductInput(product, new_value) => {
                if let Some(field) = &self.active_temporal_product_field {
                    let mut mutable_product = product;

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
                            let ignore_action = new_value.len() > mutable_product.price_input.len()
                                && mutable_product
                                    .price_input
                                    .find('.')
                                    .is_some_and(|idx| mutable_product.price_input.len() - idx > 2);

                            if !ignore_action {
                                if let Ok(num) = new_value.parse::<f32>() {
                                    mutable_product.price = Some(num);

                                    if let Some(active_product) = &mut self.active_temporal_product
                                    {
                                        active_product.price_input = new_value;
                                    }
                                } else if new_value.is_empty() {
                                    mutable_product.price = Some(0.0);

                                    if let Some(active_product) = &mut self.active_temporal_product
                                    {
                                        active_product.price_input = new_value;
                                    }
                                }
                            }
                        }
                    }

                    if let Some(pool) = &self.database {
                        action.add_task(Task::perform(
                            TemporalProduct::edit(pool.clone(), mutable_product),
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

            // Try to go up or down a page on the ProductCategories
            Message::ProductCategoriesPaginationAction(action) => match action {
                PaginationAction::Up => {
                    if self.product_categories_pagination_state.current_page > 0 {
                        self.product_categories_pagination_state.current_page -= 1;
                    }
                }
                PaginationAction::Down => {
                    let next_page_start = (self.product_categories_pagination_state.current_page
                        + 1)
                        * self.product_categories_pagination_state.items_per_page;
                    // let p_cat_len: i32 =
                    //     self.product_categories.len().try_into().unwrap_or_default();
                    // This aberration happens since adding the printpdf crate which added the deranged crate that causes this,
                    // I think I can either to this or use the line above
                    if next_page_start
                        < <usize as std::convert::TryInto<i32>>::try_into(
                            self.product_categories.len(),
                        )
                        .unwrap_or_default()
                    {
                        self.product_categories_pagination_state.current_page += 1;
                    }
                }
            },
            // Try to go up or down a page on the ProductCategoryProducts
            Message::ProductCategoryProductsPaginationAction(action) => match action {
                PaginationAction::Up => {
                    if self.product_category_products_pagination_state.current_page > 0 {
                        self.product_category_products_pagination_state.current_page -= 1;
                    }
                }
                PaginationAction::Down => {
                    let next_page_start =
                        (self.product_category_products_pagination_state.current_page + 1)
                            * self
                                .product_category_products_pagination_state
                                .items_per_page;
                    // This aberration happens since adding the printpdf crate which added the deranged crate that causes this
                    if next_page_start
                        < <usize as std::convert::TryInto<i32>>::try_into(
                            self.product_category_products
                                .as_ref()
                                .map(|v| v.len())
                                .unwrap_or(0),
                        )
                        .unwrap_or_default()
                    {
                        self.product_category_products_pagination_state.current_page += 1;
                    }
                }
            },

            // Callback after some action has been requested on the print ticket modal
            Message::PrintModalAction(modal_action) => match modal_action {
                PrintTicketModalActions::ShowModal => {
                    self.print_modal.show_modal = true;
                    action.add_task(widget::focus_next());
                }
                PrintTicketModalActions::HideModal => {
                    self.print_modal.show_modal = false;
                }
                PrintTicketModalActions::PrintTicket => {
                    // we need to get the current ticket in order to print it
                    let current_ticket = self.temporal_tickets_model.iter().find(|x| {
                        x.ticket_location
                            == match_table_location_with_number(
                                self.currently_selected_pos_state.location.clone(),
                            )
                            && x.table_id == self.currently_selected_pos_state.table_index as i32
                    });

                    if let Some(current_ticket) = current_ticket {
                        if let Some(pool) = &self.database {
                            match current_ticket.simple_invoice_id {
                                Some(invoice_id) => {
                                    // if the current ticket is already a simple invoice get it and print it
                                    action.add_task(Task::perform(
                                        SimpleInvoice::get_single(pool.clone(), invoice_id),
                                        |res| match res {
                                            Ok(invoice) => Message::PrintTicket(Box::new(invoice)),
                                            Err(err) => {
                                                eprintln!("{err}");
                                                Message::FetchTemporalTickets
                                            }
                                        },
                                    ));
                                }
                                None => {
                                    // if the current ticket is NOT already a simple invoice create it and print it
                                    action.add_task(Task::perform(
                                        SimpleInvoice::create_from_temporal_ticket(
                                            pool.clone(),
                                            current_ticket.clone(),
                                        ),
                                        |res| match res {
                                            Ok(invoice) => Message::PrintTicket(Box::new(invoice)),
                                            Err(err) => {
                                                eprintln!("{err}");
                                                Message::FetchTemporalTickets
                                            }
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }
            },
            // Updates the selected printer
            Message::UpdateSelectedPrinter(printer) => {
                self.print_modal.selected_printer = Some(printer);
            }
            // Updates the selected ticket type
            Message::UpdateSelectedTicketType(t_type) => {
                self.print_modal.ticket_type = t_type;
            }
            // Callback after creating a simple invoice from the selected temporal ticket in order to print it
            Message::PrintTicket(invoice) => {
                if let Some(p) = &self.print_modal.selected_printer {
                    let printer = Arc::new(p.clone());
                    action.add_task(Task::perform(
                        printer.print(*invoice, self.print_modal.ticket_type.clone()),
                        Message::PrintJobCompleted,
                    ));
                }
            }
            // Callback after print job is completed
            Message::PrintJobCompleted(result) => {
                if let Err(e) = result {
                    self.toasts.push(error_toast(String::from(e)));
                    eprintln!("Error: {}", e);
                }
                self.active_temporal_product = None;
                self.active_temporal_product_field = None;
                self.print_modal.ticket_type = TicketType::default();
                return self.update(Message::FetchTemporalTickets);
            }
            // Asks to unlock (delete the related invoice) of a locked ticket
            Message::UnlockTicket(ticket) => {
                if let Some(pool) = &self.database {
                    action.add_task(Task::perform(
                        SimpleInvoice::unlock_temporal_ticket(pool.clone(), ticket.clone()),
                        |res| match res {
                            Ok(_) => Message::FetchTemporalTickets,
                            Err(err) => {
                                eprintln!("{err}");
                                Message::AddToast(error_toast(err.to_string()))
                            }
                        },
                    ));
                }
            }

            // Tries to open the pay screen for the currently selected TemporalTicket
            Message::OpenPayScreen => {
                let current_ticket = self.temporal_tickets_model.iter().find(|x| {
                    x.ticket_location
                        == match_table_location_with_number(
                            self.currently_selected_pos_state.location.clone(),
                        )
                        && x.table_id == self.currently_selected_pos_state.table_index as i32
                });

                if current_ticket.is_some() {
                    self.bar_screen = BarScreen::Pay;
                }
            }
            // Changes the currently selected payment method for the given one
            Message::ChangeSelectedPaymentMethod(p_method) => {
                self.pay_screen.selected_payment_method = Some(p_method);
            }
            // Tries to execute the pay transaction for the given TemporalTicketId
            Message::PayTemporalTicket(id) => {
                // we get the id because we don't know if the ticket has been printed or not
                // so we will retrieve it by id before commiting to the pay transaction
                // this way we know if it's already a simple invoice or not
                if let Some(pool) = &self.database {
                    if let Some(payment_method) = &self.pay_screen.selected_payment_method {
                        action.add_task(Task::perform(
                            SimpleInvoice::pay_temporal_ticket(
                                pool.clone(),
                                id,
                                payment_method.id.unwrap_or_default(),
                            ),
                            |res| {
                                let mapped_result = res.map_err(|e| e.to_string());
                                Message::PaidTemporalTicket(mapped_result)
                            },
                        ));
                    }
                }
            }
            // Callback after executing the pay temporal ticket transaction
            Message::PaidTemporalTicket(res) => match res {
                Ok(_) => {
                    self.bar_screen = BarScreen::Home;
                    return self.update(Message::FetchTemporalTickets);
                }
                Err(e) => {
                    self.toasts.push(error_toast(e.to_string()));
                }
            },
        }

        action
    }
}
