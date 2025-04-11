use iced::{
    Alignment, Element, Length, Pixels,
    widget::{Column, Row, Scrollable, Space, button, column, container, pick_list, row, text},
};
use sweeten::widget::text_input;

use crate::{
    alegria::{
        utils::{
            TemporalTicketStatus, match_number_with_temporal_ticket_status,
            match_table_location_with_number,
        },
        widgets::{modal::modal, toast},
    },
    fl,
};

use super::{
    Bar, BarScreen, Message, NumPadAction, PaginationAction, PrintTicketModalActions,
    TableLocation, TicketType,
};

impl Bar {
    const GLOBAL_SPACING: f32 = 6.;
    const GLOBAL_BUTTON_HEIGHT: f32 = 60.;

    /// Returns the view of the bar screen
    pub fn view(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        let header_row = self.view_header_row();
        let page_content = match self.bar_screen {
            BarScreen::Home => self.view_bar_homescreen(),
            BarScreen::Pay => self.view_bar_payscreen(),
        };

        let content = column![header_row, page_content]
            .spacing(spacing)
            .height(Length::Fill)
            .width(Length::Fill);

        if self.print_modal.show_modal {
            let print_modal_content = container(self.view_print_modal())
                .width(700)
                .padding(30)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(container::rounded_box);

            toast::Manager::new(
                Column::new()
                    .push(modal(
                        content,
                        print_modal_content,
                        Message::PrintModalAction(PrintTicketModalActions::HideModal),
                    ))
                    .spacing(spacing)
                    .height(Length::Fill)
                    .width(Length::Fill),
                &self.toasts,
                Message::CloseToast,
            )
            .into()
        } else {
            toast::Manager::new(
                Column::new()
                    .push(content)
                    .spacing(spacing)
                    .height(Length::Fill)
                    .width(Length::Fill),
                &self.toasts,
                Message::CloseToast,
            )
            .into()
        }
    }

    //
    //  VIEW COMPOSING
    //

    const TITLE_TEXT_SIZE: f32 = 25.0;

    /// Returns the view of the header row of the bar screen
    fn view_header_row(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        let back_button = button(
            text(fl!("back"))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .on_press(Message::Back)
        .height(button_height);

        let mut header_row = row![
            back_button,
            text(fl!("bar"))
                .size(Self::TITLE_TEXT_SIZE)
                .align_y(Alignment::Center),
            Space::new(Length::Fill, Length::Shrink)
        ]
        .width(Length::Fill)
        .align_y(Alignment::Center)
        .spacing(spacing);

        let current_ticket = self.temporal_tickets_model.iter().find(|x| {
            x.ticket_location
                == match_table_location_with_number(
                    self.currently_selected_pos_state.location.clone(),
                )
                && x.table_id == self.currently_selected_pos_state.table_index as i32
        });

        if self.bar_screen == BarScreen::Home {
            if let Some(c_ticket) = current_ticket {
                if !c_ticket.products.is_empty() && c_ticket.simple_invoice_id.is_some() {
                    header_row = header_row.push(
                        button(
                            text(fl!("unlock"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::UnlockTicket(c_ticket.clone()))
                        .style(button::danger)
                        .height(button_height),
                    );
                }
                if !c_ticket.products.is_empty() {
                    header_row = header_row.push(
                        button(
                            text(fl!("print"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::PrintModalAction(
                            PrintTicketModalActions::ShowModal,
                        ))
                        .height(button_height),
                    );

                    header_row = header_row.push(
                        button(
                            text(fl!("pay"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .style(button::success)
                        .on_press(Message::OpenPayScreen)
                        .height(button_height),
                    );
                }
            }
        }

        header_row.into()
    }

    // Returns the view of the bar homescreen
    fn view_bar_homescreen(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        row![
            // LEFT SIDE COLUMN
            column![
                // UPPER LEFT SIDE
                row![
                    self.view_tables_grid(),
                    column![self.view_current_ticket_total_price(), self.view_numpad()]
                        .width(235.) //TODO: Maybe this should not be like this but the custom widget also gives some trouble
                        .spacing(spacing)
                ]
                .align_y(Alignment::Center)
                .spacing(spacing),
                // BOTTOM LEFT SIDE
                self.view_current_ticket_products()
            ]
            .spacing(spacing)
            .width(Length::Fill),
            // RIGHT SIDE ROW
            row![
                self.view_product_categories_container(),
                self.view_product_category_products_container(),
            ]
            .spacing(spacing)
            .width(Length::Fill)
        ]
        .spacing(spacing)
        .into()
    }

    // Returns the view of the bar payscreen
    fn view_bar_payscreen(&self) -> Element<Message> {
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        let current_ticket = self.temporal_tickets_model.iter().find(|x| {
            x.ticket_location
                == match_table_location_with_number(
                    self.currently_selected_pos_state.location.clone(),
                )
                && x.table_id == self.currently_selected_pos_state.table_index as i32
        });

        if let Some(c_ticket) = current_ticket {
            let total_price = {
                let mut price = 0.;
                for product in &c_ticket.products {
                    for _ in 0..product.quantity {
                        price += product.price.unwrap_or(0.);
                    }
                }

                text(format!("TOTAL: {:.2} €", price))
                    .size(25.)
                    .line_height(2.)
            };

            let print_button = button(
                text(fl!("print"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::PrintModalAction(
                PrintTicketModalActions::ShowModal,
            ))
            .height(button_height);

            let payment_methods_buttons: Vec<Element<Message>> = self
                .pay_screen
                .payment_methods
                .iter()
                .map(|p_method| {
                    button(
                        text(&p_method.name)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center),
                    )
                    .on_press(Message::ChangeSelectedPaymentMethod(p_method.clone()))
                    .style(
                        if p_method.id
                            == self.pay_screen.selected_payment_method.as_ref().unwrap().id
                        {
                            button::success
                        } else {
                            button::secondary
                        },
                    )
                    .height(button_height)
                    .into()
                })
                .collect();
            let p_methods_row = row(payment_methods_buttons).spacing(spacing);
            let p_methods_col = column![text(fl!("payment-method")), p_methods_row];

            // If selected payment method = adeudo we need to show a currently occupied reservation selector
            let reservations_selector: Element<Message> =
                if let Some(selected_p_method) = &self.pay_screen.selected_payment_method {
                    // We're hardcoding the value here which should be fine because the user
                    // can't change the values of the payment method table, but yk
                    if selected_p_method.id.unwrap_or_default().eq(&3) {
                        // TODO: Reservation selector for adeudo
                        let refresh_button = button(
                            text(fl!("refresh"))
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::FetchOccupiedReservations)
                        .height(button_height);

                        let result = column![
                            refresh_button,
                            text(format!(
                                "Number of reservations {}",
                                &self.pay_screen.occupied_reservations.len()
                            )),
                            text(format!(
                                "Number of sold rooms {}",
                                &self
                                    .pay_screen
                                    .occupied_reservations
                                    .iter()
                                    .map(|x| x.rooms.len())
                                    .sum::<usize>()
                            ))
                        ];

                        container(result).into()
                    } else {
                        container(Space::new(Length::Shrink, Length::Shrink)).into()
                    }
                } else {
                    container(Space::new(Length::Shrink, Length::Shrink)).into()
                };

            let submit_button = button(
                text(fl!("pay"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::PayTemporalTicket(c_ticket.id.unwrap_or_default()))
            .height(button_height);

            let content = row![
                column![total_price, print_button, p_methods_col, submit_button].spacing(spacing),
                reservations_selector
            ]
            .spacing(spacing);

            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(50)
                .into()
        } else {
            container(text("Error"))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        }
    }

    // Controls how many tables there are on a row
    const TABLES_PER_ROW: usize = 5;
    const NUMBER_OF_TABLES: usize = 30;

    /// Returns the view of the tables grid of the application
    fn view_tables_grid(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        let header = Row::new()
            .push(
                button(
                    text(fl!("bar"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ChangeCurrentTablesLocation(TableLocation::Bar))
                .style(|t, s| self.determine_location_button_color(t, s, TableLocation::Bar))
                .height(button_height)
                .width(Length::Fill),
            )
            .push(
                button(
                    text(fl!("restaurant"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ChangeCurrentTablesLocation(
                    TableLocation::Resturant,
                ))
                .style(|t, s| self.determine_location_button_color(t, s, TableLocation::Resturant))
                .height(button_height)
                .width(Length::Fill),
            )
            .push(
                button(
                    text(fl!("garden"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::ChangeCurrentTablesLocation(TableLocation::Garden))
                .style(|t, s| self.determine_location_button_color(t, s, TableLocation::Garden))
                .height(button_height)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .spacing(spacing);

        let mut tables_grid = Column::new().spacing(spacing).width(Length::Fill);
        let mut current_row = Row::new().spacing(spacing).width(Length::Fill);
        for index in 0..Self::NUMBER_OF_TABLES {
            let table_button = button(
                text(format!("{}", index + 1))
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .height(button_height)
            .style(move |t, s| self.determine_table_button_color(t, s, index))
            .on_press(Message::OnTableChange(index));
            current_row = current_row.push(table_button);

            if (index + 1) % Self::TABLES_PER_ROW == 0 {
                tables_grid = tables_grid.push(current_row);
                current_row = Row::new().spacing(spacing).width(Length::Fill);
            }
        }

        column![header, tables_grid]
            .width(Length::Fill)
            .spacing(spacing)
            .into()
    }

    /// Returns the view of the product categories of the bar screen
    fn view_product_categories_container(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        // Calculate the indices for the current page
        let start_index: usize = self.product_categories_pagination_state.current_page as usize
            * self.product_categories_pagination_state.items_per_page as usize;
        let end_index = usize::min(
            start_index + self.product_categories_pagination_state.items_per_page as usize,
            self.product_categories.len(),
        );

        let categories_buttons: Vec<_> = self.product_categories[start_index..end_index]
            .iter()
            .map(|category| {
                button(
                    text(category.name.as_str())
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::FetchProductCategoryProducts(category.id))
                .style(move |t, s| self.determine_product_category_button_color(t, s, category.id))
                .height(button_height)
                .width(Length::Fill)
                .into()
            })
            .collect();
        let categories_col = Column::with_children(categories_buttons)
            .spacing(spacing)
            .height(Length::Fill);

        let pagination_buttons = row![
            button(
                text(fl!("up"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::ProductCategoriesPaginationAction(
                PaginationAction::Up,
            ))
            .height(button_height)
            .width(Length::Fill),
            button(
                text(fl!("down"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::ProductCategoriesPaginationAction(
                PaginationAction::Down,
            ))
            .height(button_height)
            .width(Length::Fill)
        ]
        .spacing(spacing)
        .height(Length::Shrink);

        let result_column = column![categories_col, pagination_buttons].height(Length::Fill);

        container(result_column)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    /// Returns the view of the currently selected product category products of the bar screen
    fn view_product_category_products_container(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);
        let button_height = Length::Fixed(Self::GLOBAL_BUTTON_HEIGHT);

        // Calculate the indices for the current page
        let start_index: usize = self.product_category_products_pagination_state.current_page
            as usize
            * self
                .product_category_products_pagination_state
                .items_per_page as usize;
        let end_index = usize::min(
            start_index
                + self
                    .product_category_products_pagination_state
                    .items_per_page as usize,
            self.product_category_products
                .as_ref()
                .map(|v| v.len())
                .unwrap_or(0),
        );

        let products_buttons: Vec<_> = self
            .product_category_products
            .as_ref()
            .map(|products| {
                products[start_index..end_index]
                    .iter()
                    .map(|product| {
                        button(
                            text(product.name.as_str())
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::OnProductClicked(product.id))
                        .height(button_height)
                        .width(Length::Fill)
                        .into()
                    })
                    .collect()
            })
            .unwrap_or_default();
        let products_col = Column::with_children(products_buttons)
            .spacing(spacing)
            .height(Length::Fill);

        let pagination_buttons = row![
            button(
                text(fl!("up"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::ProductCategoryProductsPaginationAction(
                PaginationAction::Up,
            ))
            .height(button_height)
            .width(Length::Fill),
            button(
                text(fl!("down"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::ProductCategoryProductsPaginationAction(
                PaginationAction::Down,
            ))
            .height(button_height)
            .width(Length::Fill)
        ]
        .spacing(spacing)
        .height(Length::Shrink);

        let result_column = column![products_col, pagination_buttons].height(Length::Fill);

        container(result_column)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    /// Returns the view of the product (list) of the currently selected ticket
    fn view_current_ticket_products(&self) -> Element<Message> {
        let spacing = Pixels::from(Self::GLOBAL_SPACING);

        let current_ticket = &self.temporal_tickets_model.iter().find(|x| {
            x.ticket_location
                == match_table_location_with_number(
                    self.currently_selected_pos_state.location.clone(),
                )
                && x.table_id == self.currently_selected_pos_state.table_index as i32
        });

        if current_ticket.is_some() {
            let mut products_column = Column::new().spacing(spacing);

            for product in &current_ticket.unwrap().products {
                let product_quantity_str = product.quantity.to_string();

                let product_row = Row::new()
                    .push(
                        text(&product.name)
                            .size(25.)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::None),
                    )
                    .push(
                        // Only allow input if the SimpleInvoice has not been yet created
                        if current_ticket.unwrap().simple_invoice_id.is_none() {
                            text_input(&product_quantity_str, &product_quantity_str)
                                .on_focus(move |_| Message::FocusProductQuantity(product.clone()))
                                .on_input(|value| {
                                    Message::TemporalProductInput(product.clone(), value)
                                })
                                .size(25.)
                        } else {
                            text_input(&product_quantity_str, &product_quantity_str).size(25.)
                        },
                    )
                    .push(
                        // Only allow input if the SimpleInvoice has not been yet created
                        if current_ticket.unwrap().simple_invoice_id.is_none() {
                            text_input(&product.price_input, &product.price_input)
                                .on_focus(move |_| Message::FocusProductPrice(product.clone()))
                                .on_input(|value| {
                                    Message::TemporalProductInput(product.clone(), value)
                                })
                                .size(25.)
                        } else {
                            text_input(&product.price_input, &product.price_input).size(25.)
                        },
                    )
                    .spacing(spacing)
                    .align_y(Alignment::Center);

                products_column = products_column.push(product_row);
            }

            Scrollable::new(products_column).into()
        } else {
            row![
                text(fl!("no-products"))
                    .size(25.)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
            ]
            .width(Length::Fill)
            .into()
        }
    }

    /// Returns the view of the numpad
    fn view_numpad(&self) -> Element<Message> {
        crate::alegria::widgets::numpad::Numpad::new()
            .on_number_clicked(Message::OnNumpadNumberClicked)
            .on_back_clicked(Message::OnNumpadKeyClicked(NumPadAction::Erase))
            .on_delete_clicked(Message::OnNumpadKeyClicked(NumPadAction::Delete))
            .on_comma_clicked(Message::OnNumpadKeyClicked(NumPadAction::Decimal))
            .into()
    }

    /// Returns the view of the product (list) of the currently selected ticket
    fn view_current_ticket_total_price(&self) -> Element<Message> {
        let current_ticket = &self.temporal_tickets_model.iter().find(|x| {
            x.ticket_location
                == match_table_location_with_number(
                    self.currently_selected_pos_state.location.clone(),
                )
                && x.table_id == self.currently_selected_pos_state.table_index as i32
        });

        let text = if let Some(ticket) = current_ticket {
            let mut price = 0.;
            for product in &ticket.products {
                for _ in 0..product.quantity {
                    price += product.price.unwrap_or(0.);
                }
            }

            text(format!("{:.2}", price)).size(25.).line_height(2.)
        } else {
            text(fl!("unknown")).size(25.).line_height(2.)
        };

        container(text)
            .style(container::bordered_box)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    }

    /// Returns the view of the numpad
    fn view_print_modal(&self) -> Element<Message> {
        if !self.print_modal.all_printers.is_empty() {
            let spacing = Pixels::from(Self::GLOBAL_SPACING);

            let printers_label = text(fl!("printer")).width(Length::Fill);
            let printer_selector = pick_list(
                self.print_modal.all_printers.as_slice(),
                self.print_modal.selected_printer.clone(),
                Message::UpdateSelectedPrinter,
            )
            .width(Length::Fill);

            let ticket_type_label = text(fl!("ticket-type")).width(Length::Fill);
            let ticket_type_selector = pick_list(
                vec![TicketType::Invoice, TicketType::Receipt],
                Some(self.print_modal.ticket_type.clone()),
                Message::UpdateSelectedTicketType,
            )
            .width(Length::Fill);

            let submit_button = if self.print_modal.selected_printer.is_some() {
                button(
                    text(fl!("print"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .on_press(Message::PrintModalAction(
                    PrintTicketModalActions::PrintTicket,
                ))
                .width(Length::Fill)
            } else {
                button(
                    text(fl!("print"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .width(Length::Fill)
            };

            column![
                column![printers_label, printer_selector].spacing(1.),
                column![ticket_type_label, ticket_type_selector].spacing(1.),
                submit_button
            ]
            .spacing(spacing)
            .width(Length::Fill)
            .into()
        } else {
            text("No printers detected...")
                .size(25.)
                .line_height(2.)
                .into()
        }
    }

    //
    //  END OF VIEW COMPOSING
    //

    //
    // HELPERS
    //

    /// Determines the color a button of the tables grid should be given the table index, using the temporal_tickets model
    fn determine_table_button_color(
        &self,
        t: &iced::Theme,
        s: button::Status,
        t_id: usize,
    ) -> button::Style {
        let table_id = t_id as i32;

        // We have it currently selected
        if self.currently_selected_pos_state.table_index as i32 == table_id {
            match s {
                button::Status::Hovered => {
                    return button::primary(t, button::Status::Hovered);
                }
                _ => {
                    return button::primary(t, button::Status::Active);
                }
            }
        }

        let current_ticket = self.temporal_tickets_model.iter().find(|x| {
            x.table_id == table_id
                && x.ticket_location
                    == match_table_location_with_number(
                        self.currently_selected_pos_state.location.clone(),
                    )
        });

        // there is not ticket on this table
        if current_ticket.is_none() {
            match s {
                button::Status::Hovered => {
                    return button::secondary(t, button::Status::Hovered);
                }
                _ => return button::secondary(t, button::Status::Active),
            }

        // there is a pending ticket on this table (we are not currently selecting this ticket)
        } else if current_ticket.is_some_and(|y| {
            match_number_with_temporal_ticket_status(y.ticket_status)
                == TemporalTicketStatus::Pending
        }) {
            match s {
                button::Status::Hovered => {
                    return button::danger(t, button::Status::Hovered);
                }
                _ => return button::danger(t, button::Status::Active),
            }

        // there is a printed ticket on this table (we are not currently selecting this ticket)
        } else if current_ticket.is_some_and(|y| {
            match_number_with_temporal_ticket_status(y.ticket_status)
                == TemporalTicketStatus::Printed
        }) {
            match s {
                button::Status::Hovered => {
                    return button::success(t, button::Status::Hovered);
                }
                _ => return button::success(t, button::Status::Active),
            }
        }

        button::secondary(t, button::Status::Disabled)
    }

    /// Determines the color of the locations buttons using the current location of the state and given which location is which one
    fn determine_location_button_color(
        &self,
        t: &iced::Theme,
        s: button::Status,
        loc: TableLocation,
    ) -> button::Style {
        // we are currently in this location
        if loc == self.currently_selected_pos_state.location {
            match s {
                button::Status::Hovered => button::primary(t, button::Status::Hovered),
                _ => button::primary(t, button::Status::Active),
            }
        } else {
            match s {
                button::Status::Hovered => button::secondary(t, button::Status::Hovered),
                _ => button::secondary(t, button::Status::Active),
            }
        }
    }

    /// Determines the color a button of the tables grid should be given the table index, using the temporal_tickets model
    fn determine_product_category_button_color(
        &self,
        t: &iced::Theme,
        s: button::Status,
        cat_id: Option<i32>,
    ) -> button::Style {
        // we are currently selecting this category
        if self.currently_selected_product_category == cat_id {
            match s {
                button::Status::Hovered => button::primary(t, button::Status::Hovered),
                _ => button::primary(t, button::Status::Active),
            }
        } else {
            match s {
                button::Status::Hovered => button::secondary(t, button::Status::Hovered),
                _ => button::secondary(t, button::Status::Active),
            }
        }
    }

    //
    //  END OF HELPERS
    //
}
