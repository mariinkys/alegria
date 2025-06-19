// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::time::Instant;
use iced::widget::{
    Column, Row, Rule, Space, button, column, focus_next, focus_previous, pick_list, row,
    scrollable, text, text_input,
};
use iced::{Alignment, Element, Length, Renderer, Subscription, Theme, event};
use iced::{Task, widget::container};
use sqlx::{Pool, Postgres};

use crate::alegria::core::models::product::Product;
use crate::alegria::utils::styling::{
    GLOBAL_BUTTON_HEIGHT, GLOBAL_SPACING, TEXT_SIZE, TITLE_TEXT_SIZE,
};

use crate::{
    alegria::{
        core::models::product_category::ProductCategory,
        utils::pagination::{PaginationAction, PaginationConfig},
        widgets::toast::Toast,
    },
    fl,
};

pub struct Products {
    state: State,
}

enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

pub enum SubScreen {
    List {
        pagination_state: PaginationConfig,
        products: Vec<Product>,
    },
    Upsert {
        product: Box<Product>,
        product_categories: Vec<ProductCategory>,
    },
}

#[derive(Debug, Clone)]
pub enum ProductTextInputFields {
    Name,
    InsidePrice,
    OutsidePrice,
    TaxPercentage,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks the parent to go back
    Back,
    /// Asks the parent to add a toast
    AddToast(Toast),
    /// Hotkey (Subscription) pressed
    Hotkey(Hotkey),

    /// Asks to update the current list of products
    FetchProducts,
    /// Callback after initial page loading, set's the products list on the state
    PageLoaded(Vec<Product>),

    /// Try to go left or right a page
    PaginationAction(PaginationAction),

    /// Callback after asking to edit a product, searches the product on the db
    AskEditProduct(i32),
    /// Changes the upsert screen, with a default Product and grabs the product_categories (intended for calling when we need to create a new product)
    AskOpenUpsertScreen,
    /// Changes the upsert screen with the given product (we also need to get the product categories for the selector)
    OpenUpsertScreen(Box<Product>, Vec<ProductCategory>),

    /// Callback when using the text inputs to add or edit a client
    TextInputUpdate(String, ProductTextInputFields),
    /// Callback after selecting a new ProductCategoryId for the current product
    UpdatedSelectedProductCategoryId(i32),

    /// Tries to Add or Edit the current product to the database
    UpsertCurrentProduct,
    /// Callback after upserting the product  on the database
    UpsertedCurrentProduct,
    /// Tries to delete the current product
    DeleteCurrentProduct,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl Products {
    pub fn new(database: &Arc<Pool<Postgres>>) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(Product::get_all(database.clone()), |res| match res {
                Ok(res) => Message::PageLoaded(res),
                Err(err) => {
                    eprintln!("{err}");
                    Message::AddToast(Toast::error_toast(err))
                }
            }),
        )
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn update(
        &mut self,
        message: Message,
        database: &Arc<Pool<Postgres>>,
        now: Instant,
    ) -> Action {
        match message {
            Message::Back => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    match sub_screen {
                        SubScreen::List { .. } => return Action::Back,
                        SubScreen::Upsert { .. } => {
                            return self.update(Message::FetchProducts, &database.clone(), now);
                        }
                    }
                }
                Action::None
            }
            Message::AddToast(toast) => Action::AddToast(toast),
            Message::Hotkey(hotkey) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { .. } = sub_screen {
                        return match hotkey {
                            Hotkey::Tab(modifiers) => {
                                if modifiers.shift() {
                                    Action::Run(focus_previous())
                                } else {
                                    Action::Run(focus_next())
                                }
                            }
                        };
                    }
                }
                Action::None
            }
            Message::FetchProducts => Action::Run(Task::perform(
                Product::get_all(database.clone()),
                |res| match res {
                    Ok(res) => Message::PageLoaded(res),
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err))
                    }
                },
            )),
            Message::PageLoaded(res) => {
                self.state = State::Ready {
                    sub_screen: SubScreen::List {
                        pagination_state: PaginationConfig::default(),
                        products: res,
                    },
                };
                Action::None
            }
            Message::PaginationAction(pagination_action) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List {
                        products,
                        pagination_state,
                        ..
                    } = sub_screen
                    {
                        match pagination_action {
                            PaginationAction::Up => {}
                            PaginationAction::Down => {}
                            PaginationAction::Back => {
                                if pagination_state.current_page > 0 {
                                    pagination_state.current_page -= 1;
                                }
                            }
                            PaginationAction::Forward => {
                                let next_page_start = (pagination_state.current_page + 1)
                                    * pagination_state.items_per_page;
                                if next_page_start < products.len().try_into().unwrap_or_default() {
                                    pagination_state.current_page += 1;
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            Message::AskEditProduct(product_id) => {
                let database = database.clone();
                Action::Run(Task::perform(
                    async move {
                        let (product, product_categories) = tokio::join!(
                            Product::get_single(database.clone(), product_id),
                            ProductCategory::get_all(database.clone())
                        );
                        (product, product_categories)
                    },
                    |(product, product_categories)| match (product, product_categories) {
                        (Ok(product), Ok(product_categories)) => {
                            Message::OpenUpsertScreen(Box::from(product), product_categories)
                        }
                        _ => Message::AddToast(Toast::error_toast(
                            "Error fetching product or product categories",
                        )),
                    },
                ))
            }
            Message::AskOpenUpsertScreen => Action::Run(Task::perform(
                ProductCategory::get_all(database.clone()),
                |res| match res {
                    Ok(res) => Message::OpenUpsertScreen(Box::from(Product::default()), res),
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err))
                    }
                },
            )),
            Message::OpenUpsertScreen(product, product_categories) => {
                self.state = State::Ready {
                    sub_screen: SubScreen::Upsert {
                        product,
                        product_categories,
                    },
                };

                // Set a default selection on the product category
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert {
                        product,
                        product_categories,
                    } = sub_screen
                    {
                        #[warn(clippy::collapsible_if)]
                        if !product_categories.is_empty() && product.category_id.is_none() {
                            product.category_id = product_categories.first().unwrap().id;
                        }
                    }
                }
                Action::None
            }
            Message::TextInputUpdate(new_value, field) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { product, .. } = sub_screen {
                        match field {
                            ProductTextInputFields::Name => product.name = new_value,
                            ProductTextInputFields::InsidePrice => {
                                // We ignore the input if we already have two decimals and we're trying to add more
                                let ignore_action = new_value.len()
                                    > product.inside_price_input.len()
                                    && product.inside_price_input.find('.').is_some_and(|idx| {
                                        product.inside_price_input.len() - idx > 2
                                    });

                                if !ignore_action {
                                    if let Ok(num) = new_value.parse::<f32>() {
                                        product.inside_price = Some(num);
                                        product.inside_price_input = new_value;
                                    } else if new_value.is_empty() {
                                        product.inside_price = Some(0.0);
                                        product.inside_price_input = new_value;
                                    }
                                }
                            }
                            ProductTextInputFields::OutsidePrice => {
                                // We ignore the input if we already have two decimals and we're trying to add more
                                let ignore_action = new_value.len()
                                    > product.outside_price_input.len()
                                    && product.outside_price_input.find('.').is_some_and(|idx| {
                                        product.outside_price_input.len() - idx > 2
                                    });

                                if !ignore_action {
                                    if let Ok(num) = new_value.parse::<f32>() {
                                        product.outside_price = Some(num);
                                        product.outside_price_input = new_value;
                                    } else if new_value.is_empty() {
                                        product.outside_price = Some(0.0);
                                        product.outside_price_input = new_value;
                                    }
                                }
                            }
                            ProductTextInputFields::TaxPercentage => {
                                // We ignore the input if we already have two decimals and we're trying to add more
                                let ignore_action = new_value.len()
                                    > product.tax_percentage_input.len()
                                    && product.tax_percentage_input.find('.').is_some_and(|idx| {
                                        product.tax_percentage_input.len() - idx > 2
                                    });

                                if !ignore_action {
                                    if let Ok(num) = new_value.parse::<f32>() {
                                        product.tax_percentage = Some(num);
                                        product.tax_percentage_input = new_value;
                                    } else if new_value.is_empty() {
                                        product.tax_percentage = Some(0.0);
                                        product.tax_percentage_input = new_value;
                                    }
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            Message::UpdatedSelectedProductCategoryId(product_category_id) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { product, .. } = sub_screen {
                        product.category_id = Some(product_category_id)
                    }
                }
                Action::None
            }
            Message::UpsertCurrentProduct => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { product, .. } = sub_screen {
                        #[allow(clippy::collapsible_if)]
                        if product.is_valid() {
                            return match product.id {
                                Some(_id) => Action::Run(Task::perform(
                                    Product::edit(database.clone(), *product.clone()),
                                    |res| match res {
                                        Ok(_) => Message::UpsertedCurrentProduct,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::AddToast(Toast::error_toast(err))
                                        }
                                    },
                                )),
                                None => Action::Run(Task::perform(
                                    Product::add(database.clone(), *product.clone()),
                                    |res| match res {
                                        Ok(_) => Message::UpsertedCurrentProduct,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::AddToast(Toast::error_toast(err))
                                        }
                                    },
                                )),
                            };
                        }
                    }
                }
                Action::None
            }
            Message::UpsertedCurrentProduct => {
                self.update(Message::FetchProducts, &database.clone(), now)
            }
            Message::DeleteCurrentProduct => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { product, .. } = sub_screen {
                        return Action::Run(Task::perform(
                            Product::delete(database.clone(), product.id.unwrap_or_default()),
                            |res| match res {
                                Ok(_) => Message::FetchProducts,
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
        }
    }

    pub fn view(&self, _now: Instant) -> iced::Element<'_, Message> {
        match &self.state {
            State::Loading => container(text("Loading...")).center(Length::Fill).into(),
            State::Ready { sub_screen } => match sub_screen {
                SubScreen::List {
                    pagination_state,
                    products,
                } => list_screen(pagination_state, products),
                SubScreen::Upsert {
                    product,
                    product_categories,
                } => upsert_screen(product, product_categories),
            },
        }
    }

    pub fn subscription(&self, _now: Instant) -> Subscription<Message> {
        event::listen_with(handle_event)
    }
}

//
// SUBSCRIPTION HANDLING
//

#[derive(Debug, Clone)]
pub enum Hotkey {
    Tab(Modifiers),
}

fn handle_event(event: event::Event, _: event::Status, _: iced::window::Id) -> Option<Message> {
    match event {
        #[allow(clippy::collapsible_match)]
        event::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
            Key::Named(Named::Tab) => Some(Message::Hotkey(Hotkey::Tab(modifiers))),
            _ => None,
        },
        _ => None,
    }
}

//
// VIEW COMPOSING
//

// LIST SCREEN

fn list_screen<'a>(
    pagination_state: &'a PaginationConfig,
    products: &'a [Product],
) -> iced::Element<'a, Message> {
    let header = list_header();
    let grid: Element<'a, Message, Theme, Renderer> = if products.is_empty() {
        container(text(fl!("no-products")).size(TITLE_TEXT_SIZE))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding(50.)
            .into()
    } else {
        let title_row = Row::new()
            .push(
                text(fl!("name"))
                    .size(TITLE_TEXT_SIZE)
                    .width(200.)
                    .align_y(Alignment::Center),
            )
            .push(
                text(fl!("product-category"))
                    .size(TITLE_TEXT_SIZE)
                    .width(200.)
                    .align_x(Alignment::Start)
                    .align_y(Alignment::End),
            )
            .push(
                text(fl!("inside-price"))
                    .size(TITLE_TEXT_SIZE)
                    .width(200.)
                    .align_x(Alignment::Start)
                    .align_y(Alignment::End),
            )
            .push(
                text(fl!("edit"))
                    .size(TITLE_TEXT_SIZE)
                    .width(200.)
                    .align_y(Alignment::Center)
                    .align_x(Alignment::End),
            )
            .width(Length::Shrink)
            .align_y(Alignment::Center);

        // Calculate the indices for the current page
        let start_index: usize =
            pagination_state.current_page as usize * pagination_state.items_per_page as usize;
        let end_index = usize::min(
            start_index + pagination_state.items_per_page as usize,
            products.len(),
        );

        let mut grid = Column::new()
            .push(title_row)
            .spacing(GLOBAL_SPACING)
            .width(Length::Shrink);

        for product in &products[start_index..end_index] {
            let row = Row::new()
                .push(
                    text(&product.name)
                        .size(TEXT_SIZE)
                        .width(200.)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center),
                )
                .push(
                    text(&*product.product_category_name)
                        .size(TEXT_SIZE)
                        .width(200.)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center),
                )
                .push(
                    text(format!("{:.2}€", product.inside_price.unwrap_or_default()))
                        .size(TEXT_SIZE)
                        .width(200.)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center),
                )
                .push(
                    row![
                        Space::new(Length::Fill, Length::Shrink),
                        button(text(fl!("edit")).size(TEXT_SIZE).align_y(Alignment::Center))
                            .on_press(Message::AskEditProduct(product.id.unwrap()))
                            .width(Length::Shrink)
                    ]
                    .width(200.),
                )
                .align_y(Alignment::Center);

            // Limit Rule size to sum of all column widths
            grid = grid.push(row![Rule::horizontal(1.)].width(800.));
            grid = grid.push(row);
        }

        scrollable(grid).spacing(GLOBAL_SPACING).into()
    };

    let page_controls = Column::new()
        .push(row![Rule::horizontal(1.)].width(800.))
        .push(
            text(format!(
                "{} {}",
                fl!("page").as_str(),
                &pagination_state.current_page + 1
            ))
            .align_x(Alignment::Center),
        )
        .push(
            Row::new()
                .width(800.)
                .push(
                    button(
                        text(fl!("back"))
                            .center()
                            .width(Length::Fill)
                            .height(GLOBAL_BUTTON_HEIGHT),
                    )
                    .on_press(Message::PaginationAction(PaginationAction::Back)),
                )
                .push(
                    button(
                        text(fl!("next"))
                            .center()
                            .width(Length::Fill)
                            .height(GLOBAL_BUTTON_HEIGHT),
                    )
                    .on_press(Message::PaginationAction(PaginationAction::Forward)),
                )
                .align_y(Alignment::Center)
                .spacing(GLOBAL_SPACING),
        )
        .spacing(GLOBAL_SPACING)
        .align_x(Alignment::Center);

    let content = container(
        column![grid, page_controls]
            .spacing(GLOBAL_SPACING)
            .width(800.),
    )
    .width(Length::Fill)
    .align_x(Alignment::Center)
    .padding(50.);

    column![header, content]
        .spacing(GLOBAL_SPACING)
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

fn list_header<'a>() -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    let add_button = button(text(fl!("add")).center())
        .on_press(Message::AskOpenUpsertScreen)
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("products")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        add_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}

// UPSERT SCREEN

fn upsert_screen<'a>(
    product: &'a Product,
    product_categories: &'a [ProductCategory],
) -> iced::Element<'a, Message> {
    let header = upsert_header(product);

    // Name
    let name_label = text(format!("{}*", fl!("name"))).width(Length::Fill);
    let name_input = text_input(fl!("name").as_str(), &product.name)
        .on_input(|c| Message::TextInputUpdate(c, ProductTextInputFields::Name))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Produt Category
    let product_category_label = text(fl!("product-category")).width(Length::Fill);
    let selected = product_categories
        .iter()
        .find(|rt| rt.id == product.category_id);
    let product_category_selector = pick_list(product_categories, selected, |product_category| {
        Message::UpdatedSelectedProductCategoryId(product_category.id.unwrap_or_default())
    })
    .width(Length::Fill);

    // Inside Price
    let inside_price_label = text(format!("{}*", fl!("inside-price"))).width(Length::Fill);
    let inside_price_input = text_input(fl!("inside-price").as_str(), &product.inside_price_input)
        .on_input(|c| Message::TextInputUpdate(c, ProductTextInputFields::InsidePrice))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Outside Price
    let outside_price_label = text(format!("{}*", fl!("outside-price"))).width(Length::Fill);
    let outside_price_input =
        text_input(fl!("outside-price").as_str(), &product.outside_price_input)
            .on_input(|c| Message::TextInputUpdate(c, ProductTextInputFields::OutsidePrice))
            .size(TEXT_SIZE)
            .width(Length::Fill);

    // Tax Percentage
    let tax_percentage_label = text(format!("{}*", fl!("tax-percentage"))).width(Length::Fill);
    let tax_percentage_input = text_input(
        fl!("tax-percentage").as_str(),
        &product.tax_percentage_input,
    )
    .on_input(|c| Message::TextInputUpdate(c, ProductTextInputFields::TaxPercentage))
    .size(TEXT_SIZE)
    .width(Length::Fill);

    // Submit
    let submit_button_text = if product.id.is_some() {
        text(fl!("edit"))
    } else {
        text(fl!("add"))
    };
    let submit_button = button(submit_button_text.center().size(TEXT_SIZE))
        .on_press_maybe(product.is_valid().then_some(Message::UpsertCurrentProduct))
        .width(Length::Fill);

    // Input Columns
    let name_input_column = column![name_label, name_input].width(850.).spacing(1.);
    let product_category_column = column![product_category_label, product_category_selector]
        .width(850.)
        .spacing(1.);
    let inside_price_input_column = column![inside_price_label, inside_price_input]
        .width(850.)
        .spacing(1.);
    let outside_price_input_column = column![outside_price_label, outside_price_input]
        .width(850.)
        .spacing(1.);
    let tax_input_column = column![tax_percentage_label, tax_percentage_input]
        .width(850.)
        .spacing(1.);

    let form_column = Column::new()
        .push(name_input_column)
        .push(product_category_column)
        .push(inside_price_input_column)
        .push(outside_price_input_column)
        .push(tax_input_column)
        .push(submit_button)
        .width(850.)
        .spacing(GLOBAL_SPACING);

    column![
        header,
        container(form_column)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .padding(50.)
    ]
    .into()
}

fn upsert_header<'a>(product: &'a Product) -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    let delete_button = button(text(fl!("delete")).center())
        .style(button::danger)
        .on_press_maybe(product.id.map(|_| Message::DeleteCurrentProduct))
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("products")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        delete_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}
