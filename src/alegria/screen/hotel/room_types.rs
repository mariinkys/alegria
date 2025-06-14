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

use crate::alegria::utils::styling::{
    GLOBAL_BUTTON_HEIGHT, GLOBAL_SPACING, TEXT_SIZE, TITLE_TEXT_SIZE,
};
// SPDX-License-Identifier: GPL-3.0-only
use crate::{
    alegria::{
        core::models::room_type::RoomType,
        utils::pagination::{PaginationAction, PaginationConfig},
        widgets::toast::Toast,
    },
    fl,
};

pub struct RoomTypes {
    state: State,
}

enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

pub enum SubScreen {
    List {
        pagination_state: PaginationConfig,
        room_types: Vec<RoomType>,
    },
    Upsert {
        room_type: Box<RoomType>,
    },
}

#[derive(Debug, Clone)]
pub enum RoomTypeTextInputFields {
    Name,
    Price,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Asks the parent to go back
    Back,
    /// Asks the parent to add a toast
    AddToast(Toast),
    /// Hotkey (Subscription) pressed
    Hotkey(Hotkey),

    /// Asks to update the current list of room types
    FetchRoomTypes,
    /// Callback after initial page loading, set's the room tpyes list on the state
    PageLoaded(Vec<RoomType>),

    /// Try to go left or right a page
    PaginationAction(PaginationAction),

    /// Callback after asking to edit a room_type, searches the room_type on the db
    AskEditRoomType(i32),
    /// Changes the upsert screen with the given room type
    OpenUpsertScreen(Box<RoomType>),

    /// Callback when using the text inputs to add or edit a client
    TextInputUpdate(String, RoomTypeTextInputFields),

    /// Tries to Add or Edit the current room_type to the database
    UpsertCurrentRoomType,
    /// Callback after upserting the room type on the database
    UpsertedCurrentRoomType,
    /// Tries to delete the current room type
    DeleteCurrentRoomType,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl RoomTypes {
    pub fn new(database: &Arc<Pool<Postgres>>) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(RoomType::get_all(database.clone()), |res| match res {
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
                            return self.update(Message::FetchRoomTypes, &database.clone(), now);
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
            Message::FetchRoomTypes => Action::Run(Task::perform(
                RoomType::get_all(database.clone()),
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
                        room_types: res,
                    },
                };
                Action::None
            }
            Message::PaginationAction(pagination_action) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List {
                        room_types,
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
                                if next_page_start < room_types.len().try_into().unwrap_or_default()
                                {
                                    pagination_state.current_page += 1;
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            Message::AskEditRoomType(room_type_id) => Action::Run(Task::perform(
                RoomType::get_single(database.clone(), room_type_id),
                |res| match res {
                    Ok(res) => Message::OpenUpsertScreen(Box::from(res)),
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err))
                    }
                },
            )),
            Message::OpenUpsertScreen(room_type) => {
                self.state = State::Ready {
                    sub_screen: SubScreen::Upsert { room_type },
                };
                Action::None
            }
            Message::TextInputUpdate(new_value, field) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { room_type, .. } = sub_screen {
                        match field {
                            RoomTypeTextInputFields::Name => room_type.name = new_value,
                            RoomTypeTextInputFields::Price => {
                                // We ignore the input if we already have two decimals and we're trying to add more
                                let ignore_action = new_value.len() > room_type.price_input.len()
                                    && room_type
                                        .price_input
                                        .find('.')
                                        .is_some_and(|idx| room_type.price_input.len() - idx > 2);

                                if !ignore_action {
                                    if let Ok(num) = new_value.parse::<f32>() {
                                        room_type.price = Some(num);
                                        room_type.price_input = new_value;
                                    } else if new_value.is_empty() {
                                        room_type.price = Some(0.0);
                                        room_type.price_input = new_value;
                                    }
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            Message::UpsertCurrentRoomType => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { room_type, .. } = sub_screen {
                        #[allow(clippy::collapsible_if)]
                        if room_type.is_valid() {
                            return match room_type.id {
                                Some(_id) => Action::Run(Task::perform(
                                    RoomType::edit(database.clone(), *room_type.clone()),
                                    |res| match res {
                                        Ok(_) => Message::UpsertedCurrentRoomType,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::AddToast(Toast::error_toast(err))
                                        }
                                    },
                                )),
                                None => Action::Run(Task::perform(
                                    RoomType::add(database.clone(), *room_type.clone()),
                                    |res| match res {
                                        Ok(_) => Message::UpsertedCurrentRoomType,
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
            Message::UpsertedCurrentRoomType => {
                self.update(Message::FetchRoomTypes, &database.clone(), now)
            }
            Message::DeleteCurrentRoomType => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { room_type, .. } = sub_screen {
                        return Action::Run(Task::perform(
                            RoomType::delete(database.clone(), room_type.id.unwrap_or_default()),
                            |res| match res {
                                Ok(_) => Message::FetchRoomTypes,
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
                    room_types,
                } => list_screen(pagination_state, room_types),
                SubScreen::Upsert { room_type } => upsert_screen(room_type),
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
    room_types: &'a [RoomType],
) -> iced::Element<'a, Message> {
    let header = list_header();
    let grid: Element<'a, Message, Theme, Renderer> = if room_types.is_empty() {
        container(text(fl!("no-room-types")).size(TITLE_TEXT_SIZE))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .padding(50.)
            .into()
    } else {
        let title_row = Row::new()
            .push(
                text(fl!("name"))
                    .size(TITLE_TEXT_SIZE)
                    .width(300.)
                    .align_y(Alignment::Center),
            )
            .push(
                text(fl!("price"))
                    .size(TITLE_TEXT_SIZE)
                    .width(300.)
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
            room_types.len(),
        );

        let mut grid = Column::new()
            .push(title_row)
            .spacing(GLOBAL_SPACING)
            .width(Length::Shrink);

        for room_type in &room_types[start_index..end_index] {
            let row = Row::new()
                .push(
                    text(&room_type.name)
                        .size(TEXT_SIZE)
                        .width(300.)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center),
                )
                .push(
                    text(format!("{:.2} €", room_type.price.unwrap_or(0.)))
                        .size(TEXT_SIZE)
                        .width(300.)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center),
                )
                .push(
                    row![
                        Space::new(Length::Fill, Length::Shrink),
                        button(text(fl!("edit")).size(TEXT_SIZE).align_y(Alignment::Center))
                            .on_press(Message::AskEditRoomType(room_type.id.unwrap()))
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
        .on_press(Message::OpenUpsertScreen(Box::from(RoomType::default())))
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("room-types")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        add_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}

// UPSERT SCREEN

fn upsert_screen<'a>(room_type: &'a RoomType) -> iced::Element<'a, Message> {
    let header = upsert_header(room_type);

    // Name
    let name_label = text(format!("{}*", fl!("name"))).width(Length::Fill);
    let name_input = text_input(fl!("name").as_str(), &room_type.name)
        .on_input(|c| Message::TextInputUpdate(c, RoomTypeTextInputFields::Name))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Price
    let price_label = text(format!("{}*", fl!("price"))).width(Length::Fill);
    let price_input = text_input(fl!("price").as_str(), &room_type.price_input)
        .on_input(|c| Message::TextInputUpdate(c, RoomTypeTextInputFields::Price))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    // Submit
    let submit_button_text = if room_type.id.is_some() {
        text(fl!("edit"))
    } else {
        text(fl!("add"))
    };
    let submit_button = button(submit_button_text.center().size(TEXT_SIZE))
        .on_press_maybe(
            room_type
                .is_valid()
                .then_some(Message::UpsertCurrentRoomType),
        )
        .width(Length::Fill);

    // Input Columns
    let name_input_column = column![name_label, name_input].width(850.).spacing(1.);
    let price_input_column = column![price_label, price_input].width(850.).spacing(1.);

    let form_column = Column::new()
        .push(name_input_column)
        .push(price_input_column)
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

fn upsert_header<'a>(room_type: &'a RoomType) -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    let delete_button = button(text(fl!("delete")).center())
        .style(button::danger)
        .on_press_maybe(room_type.id.map(|_| Message::DeleteCurrentRoomType))
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("room-type")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        delete_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}
