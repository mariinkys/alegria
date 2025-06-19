// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::keyboard::key::Named;
use iced::keyboard::{self, Key, Modifiers};
use iced::time::Instant;
use iced::widget::{
    Column, Row, Rule, Space, button, column, focus_next, focus_previous, row, scrollable, text,
    text_input,
};
use iced::{Alignment, Element, Length, Renderer, Subscription, Theme, event};
use iced::{Task, widget::container};
use sqlx::{Pool, Postgres};

use alegria_core::models::product_category::ProductCategory;
use alegria_utils::pagination::*;
use alegria_utils::styling::{GLOBAL_BUTTON_HEIGHT, GLOBAL_SPACING, TEXT_SIZE, TITLE_TEXT_SIZE};

use crate::{alegria::widgets::toast::Toast, fl};

pub struct ProductCategories {
    state: State,
}

enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

pub enum SubScreen {
    List {
        pagination_state: PaginationConfig,
        product_categories: Vec<ProductCategory>,
    },
    Upsert {
        product_category: Box<ProductCategory>,
    },
}

#[derive(Debug, Clone)]
pub enum ProductCategoryTextInputFields {
    Name,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks the parent to go back
    Back,
    /// Asks the parent to add a toast
    AddToast(Toast),
    /// Hotkey (Subscription) pressed
    Hotkey(Hotkey),

    /// Asks to update the current list of product-categories
    FetchProductCategories,
    /// Callback after initial page loading, set's the room tpyes list on the state
    PageLoaded(Vec<ProductCategory>),

    /// Try to go left or right a page
    PaginationAction(PaginationAction),

    /// Callback after asking to edit a product_category, searches the product_category on the db
    AskEditProductCategory(i32),
    /// Changes the upsert screen with the given product-category
    OpenUpsertScreen(Box<ProductCategory>),

    /// Callback when using the text inputs to add or edit a client
    TextInputUpdate(String, ProductCategoryTextInputFields),

    /// Tries to Add or Edit the current product_category to the database
    UpsertCurrentProductCategory,
    /// Callback after upserting the product-category on the database
    UpsertedCurrentProductCategory,
    /// Tries to delete the current product-category
    DeleteCurrentProductCategory,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl ProductCategories {
    pub fn new(database: &Arc<Pool<Postgres>>) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(
                ProductCategory::get_all(database.clone()),
                |res| match res {
                    Ok(res) => Message::PageLoaded(res),
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err))
                    }
                },
            ),
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
                            return self.update(
                                Message::FetchProductCategories,
                                &database.clone(),
                                now,
                            );
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
            Message::FetchProductCategories => Action::Run(Task::perform(
                ProductCategory::get_all(database.clone()),
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
                        product_categories: res,
                    },
                };
                Action::None
            }
            Message::PaginationAction(pagination_action) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List {
                        product_categories,
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
                                if next_page_start
                                    < product_categories.len().try_into().unwrap_or_default()
                                {
                                    pagination_state.current_page += 1;
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            Message::AskEditProductCategory(product_category_id) => Action::Run(Task::perform(
                ProductCategory::get_single(database.clone(), product_category_id),
                |res| match res {
                    Ok(res) => Message::OpenUpsertScreen(Box::from(res)),
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err))
                    }
                },
            )),
            Message::OpenUpsertScreen(product_category) => {
                self.state = State::Ready {
                    sub_screen: SubScreen::Upsert { product_category },
                };
                Action::None
            }
            Message::TextInputUpdate(new_value, field) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert {
                        product_category, ..
                    } = sub_screen
                    {
                        match field {
                            ProductCategoryTextInputFields::Name => {
                                product_category.name = new_value
                            }
                        }
                    }
                }
                Action::None
            }
            Message::UpsertCurrentProductCategory => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert {
                        product_category, ..
                    } = sub_screen
                    {
                        #[allow(clippy::collapsible_if)]
                        if product_category.is_valid() {
                            return match product_category.id {
                                Some(_id) => Action::Run(Task::perform(
                                    ProductCategory::edit(
                                        database.clone(),
                                        *product_category.clone(),
                                    ),
                                    |res| match res {
                                        Ok(_) => Message::UpsertedCurrentProductCategory,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::AddToast(Toast::error_toast(err))
                                        }
                                    },
                                )),
                                None => Action::Run(Task::perform(
                                    ProductCategory::add(
                                        database.clone(),
                                        *product_category.clone(),
                                    ),
                                    |res| match res {
                                        Ok(_) => Message::UpsertedCurrentProductCategory,
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
            Message::UpsertedCurrentProductCategory => {
                self.update(Message::FetchProductCategories, &database.clone(), now)
            }
            Message::DeleteCurrentProductCategory => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert {
                        product_category, ..
                    } = sub_screen
                    {
                        return Action::Run(Task::perform(
                            ProductCategory::delete(
                                database.clone(),
                                product_category.id.unwrap_or_default(),
                            ),
                            |res| match res {
                                Ok(_) => Message::FetchProductCategories,
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
                    product_categories,
                } => list_screen(pagination_state, product_categories),
                SubScreen::Upsert { product_category } => upsert_screen(product_category),
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
    product_categories: &'a [ProductCategory],
) -> iced::Element<'a, Message> {
    let header = list_header();
    let grid: Element<'a, Message, Theme, Renderer> = if product_categories.is_empty() {
        container(text(fl!("no-product-categories")).size(TITLE_TEXT_SIZE))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding(50.)
            .into()
    } else {
        let title_row = Row::new()
            .push(
                text(fl!("name"))
                    .size(TITLE_TEXT_SIZE)
                    .width(600.)
                    .align_y(Alignment::Center),
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
            product_categories.len(),
        );

        let mut grid = Column::new()
            .push(title_row)
            .spacing(GLOBAL_SPACING)
            .width(Length::Shrink);

        for product_category in &product_categories[start_index..end_index] {
            let row = Row::new()
                .push(
                    text(&product_category.name)
                        .size(TEXT_SIZE)
                        .width(600.)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center),
                )
                .push(
                    row![
                        Space::new(Length::Fill, Length::Shrink),
                        button(text(fl!("edit")).size(TEXT_SIZE).align_y(Alignment::Center))
                            .on_press(Message::AskEditProductCategory(
                                product_category.id.unwrap()
                            ))
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
        .on_press(Message::OpenUpsertScreen(Box::from(
            ProductCategory::default(),
        )))
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("product-categories")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        add_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}

// UPSERT SCREEN

fn upsert_screen<'a>(product_category: &'a ProductCategory) -> iced::Element<'a, Message> {
    let header = upsert_header(product_category);

    // Name
    let name_label = text(format!("{}*", fl!("name"))).width(Length::Fill);
    let name_input = text_input(fl!("name").as_str(), &product_category.name)
        .on_input(|c| Message::TextInputUpdate(c, ProductCategoryTextInputFields::Name))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Submit
    let submit_button_text = if product_category.id.is_some() {
        text(fl!("edit"))
    } else {
        text(fl!("add"))
    };
    let submit_button = button(submit_button_text.center().size(TEXT_SIZE))
        .on_press_maybe(
            product_category
                .is_valid()
                .then_some(Message::UpsertCurrentProductCategory),
        )
        .width(Length::Fill);

    // Input Columns
    let name_input_column = column![name_label, name_input].width(850.).spacing(1.);

    let form_column = Column::new()
        .push(name_input_column)
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

fn upsert_header<'a>(product_category: &'a ProductCategory) -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    let delete_button = button(text(fl!("delete")).center())
        .style(button::danger)
        .on_press_maybe(
            product_category
                .id
                .map(|_| Message::DeleteCurrentProductCategory),
        )
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("product-category")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        delete_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}
