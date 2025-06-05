use crate::{
    alegria::{
        core::models::{
            product::Product, product_category::ProductCategory, temporal_ticket::TemporalTicket,
        },
        screen::{
            Bar,
            bar::{
                ActiveTemporalProduct, BarPagination, CurrentPosition, Message, NumPadAction,
                PaginationAction, State, SubScreen, TableLocation,
                match_table_location_with_number,
            },
        },
        utils::{TemporalTicketStatus, match_number_with_temporal_ticket_status},
        widgets::focusable_text_input::TextInput,
    },
    fl,
};
use iced::{
    Alignment, Length, Pixels,
    time::Instant,
    widget::{Column, Row, Scrollable, Space, button, column, container, row, text},
};

const GLOBAL_SPACING: f32 = 6.;
const GLOBAL_BUTTON_HEIGHT: f32 = 60.;
const TITLE_TEXT_SIZE: f32 = 25.0;

// Tables Grid
const TABLES_PER_ROW: usize = 5;
const NUMBER_OF_TABLES: usize = 30;

impl Bar {
    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading...")).center(Length::Fill).into(),
            State::Ready { sub_screen } => match sub_screen {
                SubScreen::Bar {
                    temporal_tickets,
                    product_categories,
                    product_category_products,
                    pagination,
                    current_position,
                    active_temporal_product,
                } => container(bar_view(
                    temporal_tickets,
                    product_categories,
                    product_category_products,
                    pagination,
                    current_position,
                    active_temporal_product,
                ))
                .center(Length::Fill)
                .into(),
                SubScreen::Pay => todo!(),
            },
        }
    }
}

/// View of the bar subscreen
fn bar_view<'a>(
    temporal_tickets: &'a [TemporalTicket],
    product_categories: &'a [ProductCategory],
    product_category_products: &'a Option<Vec<Product>>,
    pagination: &'a BarPagination,
    current_position: &'a CurrentPosition,
    _active_temporal_product: &'a ActiveTemporalProduct,
) -> iced::Element<'a, Message> {
    let spacing = Pixels::from(GLOBAL_SPACING);

    let header = bar_header(temporal_tickets, current_position);
    let content = row![
        // LEFT SIDE COLUMN
        column![
            // UPPER LEFT SIDE
            row![
                tables_grid(temporal_tickets, current_position),
                column![
                    total_ticket_price(temporal_tickets, current_position),
                    numpad()
                ]
                .width(235.) //TODO: Maybe this should not be like this but the custom widget also gives some trouble
                .spacing(spacing)
            ]
            .align_y(Alignment::Center)
            .spacing(spacing),
            // BOTTOM LEFT SIDE
            current_ticket_products(temporal_tickets, current_position)
        ]
        .spacing(spacing)
        .width(Length::Fill),
        // RIGHT SIDE ROW
        row![
            product_categories_container(product_categories, pagination, current_position),
            product_category_products_container(product_category_products, pagination),
        ]
        .spacing(spacing)
        .width(Length::Fill)
    ]
    .spacing(spacing);

    column![header, content].padding(3.).spacing(spacing).into()
}

/// Returns the view of the header row of the bar screen
fn bar_header<'a>(
    temporal_tickets: &'a [TemporalTicket],
    current_position: &'a CurrentPosition,
) -> iced::Element<'a, Message> {
    let spacing = Pixels::from(GLOBAL_SPACING);
    let button_height = Length::Fixed(GLOBAL_BUTTON_HEIGHT);

    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(button_height);

    let mut header_row = row![
        back_button,
        text(fl!("bar"))
            .size(TITLE_TEXT_SIZE)
            .align_y(Alignment::Center),
        Space::new(Length::Fill, Length::Shrink)
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .spacing(spacing);

    let current_ticket = temporal_tickets.iter().find(|x| {
        x.ticket_location
            == super::match_table_location_with_number(&current_position.table_location)
            && x.table_id == current_position.table_index
    });

    if let Some(c_ticket) = current_ticket {
        if !c_ticket.products.is_empty() && c_ticket.simple_invoice_id.is_some() {
            header_row = header_row.push(
                button(
                    text(fl!("unlock"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                //.on_press(Message::UnlockTicket(c_ticket.clone()))
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
                // .on_press(Message::PrintModalAction(
                //     PrintTicketModalActions::ShowModal,
                // ))
                .height(button_height),
            );

            header_row = header_row.push(
                button(
                    text(fl!("pay"))
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                )
                .style(button::success)
                //.on_press(Message::OpenPayScreen)
                .height(button_height),
            );
        }
    }

    header_row.into()
}

/// Returns the view of the header row of the bar screen
fn tables_grid<'a>(
    temporal_tickets: &'a [TemporalTicket],
    current_position: &'a CurrentPosition,
) -> iced::Element<'a, Message> {
    let spacing = Pixels::from(GLOBAL_SPACING);
    let button_height = Length::Fixed(GLOBAL_BUTTON_HEIGHT);

    let header = Row::new()
        .push(
            button(
                text(fl!("bar"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            //.on_press(Message::ChangeCurrentTablesLocation(TableLocation::Bar))
            .style(|t, s| {
                determine_location_button_color(current_position, t, s, TableLocation::Bar)
            })
            .height(button_height)
            .width(Length::Fill),
        )
        .push(
            button(
                text(fl!("restaurant"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            // .on_press(Message::ChangeCurrentTablesLocation(
            //     TableLocation::Resturant,
            // ))
            .style(|t, s| {
                determine_location_button_color(current_position, t, s, TableLocation::Resturant)
            })
            .height(button_height)
            .width(Length::Fill),
        )
        .push(
            button(
                text(fl!("garden"))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            //.on_press(Message::ChangeCurrentTablesLocation(TableLocation::Garden))
            .style(|t, s| {
                determine_location_button_color(current_position, t, s, TableLocation::Garden)
            })
            .height(button_height)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .spacing(spacing);

    let mut tables_grid = Column::new().spacing(spacing).width(Length::Fill);
    let mut current_row = Row::new().spacing(spacing).width(Length::Fill);
    for index in 0..NUMBER_OF_TABLES {
        let table_button = button(
            text(format!("{}", index + 1))
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .height(button_height)
        .style(move |t, s| {
            determine_table_button_color(current_position, temporal_tickets, t, s, index)
        });
        //.on_press(Message::OnTableChange(index));
        current_row = current_row.push(table_button);

        if (index + 1) % TABLES_PER_ROW == 0 {
            tables_grid = tables_grid.push(current_row);
            current_row = Row::new().spacing(spacing).width(Length::Fill);
        }
    }

    column![header, tables_grid]
        .width(Length::Fill)
        .spacing(spacing)
        .into()
}

/// Returns the view of the product (list) of the currently selected ticket
fn total_ticket_price<'a>(
    temporal_tickets: &'a [TemporalTicket],
    current_position: &'a CurrentPosition,
) -> iced::Element<'a, Message> {
    let current_ticket = temporal_tickets.iter().find(|x| {
        x.ticket_location
            == super::match_table_location_with_number(&current_position.table_location)
            && x.table_id == current_position.table_index
    });

    let text = if let Some(ticket) = current_ticket {
        let mut price = 0.;
        for product in &ticket.products {
            for _ in 0..product.quantity {
                price += product.price.unwrap_or(0.);
            }
        }

        text(format!("{price:.2}")).size(25.).line_height(2.)
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
fn numpad<'a>() -> iced::Element<'a, Message> {
    crate::alegria::widgets::numpad::Numpad::new()
        .on_number_clicked(Message::OnNumpadNumberClicked)
        .on_back_clicked(Message::OnNumpadKeyClicked(NumPadAction::Erase))
        .on_delete_clicked(Message::OnNumpadKeyClicked(NumPadAction::Delete))
        .on_comma_clicked(Message::OnNumpadKeyClicked(NumPadAction::Decimal))
        .into()
}

/// Returns the view of the product (list) of the currently selected ticket
fn current_ticket_products<'a>(
    temporal_tickets: &'a [TemporalTicket],
    current_position: &'a CurrentPosition,
) -> iced::Element<'a, Message> {
    let spacing = Pixels::from(GLOBAL_SPACING);

    let current_ticket = temporal_tickets.iter().find(|x| {
        x.ticket_location
            == super::match_table_location_with_number(&current_position.table_location)
            && x.table_id == current_position.table_index
    });

    if let Some(current_ticket) = current_ticket {
        let mut products_column = Column::new().spacing(spacing);

        for product in &current_ticket.products {
            let product_quantity_str = product.quantity.to_string();

            let quantity_input = TextInput::new(&product_quantity_str, &product_quantity_str)
                .on_focus(move |_| {
                    Message::FocusTemporalProduct(
                        product.clone(),
                        super::TemporalProductField::Quantity,
                    )
                })
                .on_input_maybe(if current_ticket.simple_invoice_id.is_none() {
                    Some(|value| Message::TemporalProductInput(product.clone(), value))
                } else {
                    None
                })
                .size(25.);

            let price_input = TextInput::new(&product.price_input, &product.price_input)
                .on_focus(move |_| {
                    Message::FocusTemporalProduct(
                        product.clone(),
                        super::TemporalProductField::Price,
                    )
                })
                .on_input_maybe(if current_ticket.simple_invoice_id.is_none() {
                    Some(|value| Message::TemporalProductInput(product.clone(), value))
                } else {
                    None
                })
                .size(25.);

            let product_row = Row::new()
                .push(
                    text(&product.name)
                        .size(25.)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::None),
                )
                .push(quantity_input)
                .push(price_input)
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

fn product_categories_container<'a>(
    product_categories: &'a [ProductCategory],
    pagination: &'a BarPagination,
    current_position: &'a CurrentPosition,
) -> iced::Element<'a, Message> {
    let spacing = Pixels::from(GLOBAL_SPACING);
    let button_height = Length::Fixed(GLOBAL_BUTTON_HEIGHT);

    // Calculate the indices for the current page
    let start_index: usize = pagination.product_categories.current_page as usize
        * pagination.product_categories.items_per_page as usize;
    let end_index = usize::min(
        start_index + pagination.product_categories.items_per_page as usize,
        product_categories.len(),
    );

    let categories_buttons: Vec<_> = product_categories[start_index..end_index]
        .iter()
        .map(|category| {
            button(
                text(category.name.as_str())
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
            )
            .on_press(Message::FetchProductCategoryProducts(category.id))
            .style(move |t, s| {
                determine_product_category_button_color(current_position, t, s, category.id)
            })
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

fn product_category_products_container<'a>(
    product_category_products: &'a Option<Vec<Product>>,
    pagination: &'a BarPagination,
) -> iced::Element<'a, Message> {
    let spacing = Pixels::from(GLOBAL_SPACING);
    let button_height = Length::Fixed(GLOBAL_BUTTON_HEIGHT);

    // Calculate the indices for the current page
    let start_index: usize = pagination.product_category_products.current_page as usize
        * pagination.product_category_products.items_per_page as usize;
    let end_index = usize::min(
        start_index + pagination.product_category_products.items_per_page as usize,
        product_category_products
            .as_ref()
            .map(|v| v.len())
            .unwrap_or(0),
    );

    let products_buttons: Vec<_> = product_category_products
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
                    //.on_press(Message::OnProductClicked(product.id))
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

//
// HELPERS
//

/// Determines the color a button of the tables grid should be given the table index, using the temporal_tickets model
fn determine_table_button_color(
    current_position: &CurrentPosition,
    temporal_tickets: &[TemporalTicket],
    t: &iced::Theme,
    s: button::Status,
    t_id: usize,
) -> button::Style {
    let table_id = t_id as i32;

    // We have it currently selected
    if current_position.table_index == table_id {
        match s {
            button::Status::Hovered => {
                return button::primary(t, button::Status::Hovered);
            }
            _ => {
                return button::primary(t, button::Status::Active);
            }
        }
    }

    let current_ticket = temporal_tickets.iter().find(|x| {
        x.table_id == table_id
            && x.ticket_location
                == match_table_location_with_number(&current_position.table_location)
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
        match_number_with_temporal_ticket_status(y.ticket_status) == TemporalTicketStatus::Pending
    }) {
        match s {
            button::Status::Hovered => {
                return button::danger(t, button::Status::Hovered);
            }
            _ => return button::danger(t, button::Status::Active),
        }

    // there is a printed ticket on this table (we are not currently selecting this ticket)
    } else if current_ticket.is_some_and(|y| {
        match_number_with_temporal_ticket_status(y.ticket_status) == TemporalTicketStatus::Printed
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
    current_position: &CurrentPosition,
    t: &iced::Theme,
    s: button::Status,
    loc: TableLocation,
) -> button::Style {
    // we are currently in this location
    if loc == current_position.table_location {
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
    current_position: &CurrentPosition,
    t: &iced::Theme,
    s: button::Status,
    cat_id: Option<i32>,
) -> button::Style {
    // we are currently selecting this category
    if current_position.selected_product_category == cat_id {
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
// END OF HELPERS
//
