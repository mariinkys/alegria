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

use crate::alegria::core::models::room::Room;
use crate::alegria::utils::styling::{
    GLOBAL_BUTTON_HEIGHT, GLOBAL_SPACING, TEXT_SIZE, TITLE_TEXT_SIZE,
};

use crate::{
    alegria::{
        core::models::room_type::RoomType,
        utils::pagination::{PaginationAction, PaginationConfig},
        widgets::toast::Toast,
    },
    fl,
};

pub struct Rooms {
    state: State,
}

enum State {
    Loading,
    Ready { sub_screen: SubScreen },
}

pub enum SubScreen {
    List {
        pagination_state: PaginationConfig,
        rooms: Vec<Room>,
    },
    Upsert {
        room: Box<Room>,
        room_types: Vec<RoomType>,
    },
}

#[derive(Debug, Clone)]
pub enum RoomTextInputFields {
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

    /// Asks to update the current list of rooms
    FetchRooms,
    /// Callback after initial page loading, set's the rooms list on the state
    PageLoaded(Vec<Room>),

    /// Try to go left or right a page
    PaginationAction(PaginationAction),

    /// Callback after asking to edit a room, searches the room on the db
    AskEditRoom(i32),
    /// Changes the upsert screen, with a default Room and grabs the room_types (intended for calling when we need to create a new room)
    AskOpenUpsertScreen,
    /// Changes the upsert screen with the given room (we also need to get the room types for the selector)
    OpenUpsertScreen(Box<Room>, Vec<RoomType>),

    /// Callback when using the text inputs to add or edit a client
    TextInputUpdate(String, RoomTextInputFields),
    /// Callback after selecting a new RoomTypeId for the current room
    UpdatedSelectedRoomTypeId(i32),

    /// Tries to Add or Edit the current room to the database
    UpsertCurrentRoom,
    /// Callback after upserting the room  on the database
    UpsertedCurrentRoom,
    /// Tries to delete the current room
    DeleteCurrentRoom,
}

pub enum Action {
    None,
    Back,
    Run(Task<Message>),
    AddToast(Toast),
}

impl Rooms {
    pub fn new(database: &Arc<Pool<Postgres>>) -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading,
            },
            Task::perform(Room::get_all(database.clone()), |res| match res {
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
                            return self.update(Message::FetchRooms, &database.clone(), now);
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
            Message::FetchRooms => Action::Run(Task::perform(
                Room::get_all(database.clone()),
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
                        rooms: res,
                    },
                };
                Action::None
            }
            Message::PaginationAction(pagination_action) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::List {
                        rooms,
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
                                if next_page_start < rooms.len().try_into().unwrap_or_default() {
                                    pagination_state.current_page += 1;
                                }
                            }
                        }
                    }
                }
                Action::None
            }
            Message::AskEditRoom(room_id) => {
                let database = database.clone();
                Action::Run(Task::perform(
                    async move {
                        let (room, room_types) = tokio::join!(
                            Room::get_single(database.clone(), room_id),
                            RoomType::get_all(database.clone())
                        );
                        (room, room_types)
                    },
                    |(room, room_types)| match (room, room_types) {
                        (Ok(room), Ok(room_types)) => {
                            Message::OpenUpsertScreen(Box::from(room), room_types)
                        }
                        _ => Message::AddToast(Toast::error_toast(
                            "Error fetching room or room types",
                        )),
                    },
                ))
            }
            Message::AskOpenUpsertScreen => Action::Run(Task::perform(
                RoomType::get_all(database.clone()),
                |res| match res {
                    Ok(res) => Message::OpenUpsertScreen(Box::from(Room::default()), res),
                    Err(err) => {
                        eprintln!("{err}");
                        Message::AddToast(Toast::error_toast(err))
                    }
                },
            )),
            Message::OpenUpsertScreen(room, room_types) => {
                self.state = State::Ready {
                    sub_screen: SubScreen::Upsert { room, room_types },
                };

                // Set a default selection on the room type
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { room, room_types } = sub_screen {
                        #[warn(clippy::collapsible_if)]
                        if !room_types.is_empty() {
                            room.room_type_id = room_types.first().unwrap().id;
                        }
                    }
                }
                Action::None
            }
            Message::TextInputUpdate(new_value, field) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { room, .. } = sub_screen {
                        match field {
                            RoomTextInputFields::Name => room.name = new_value,
                        }
                    }
                }
                Action::None
            }
            Message::UpdatedSelectedRoomTypeId(room_type_id) => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { room, .. } = sub_screen {
                        room.room_type_id = Some(room_type_id)
                    }
                }
                Action::None
            }
            Message::UpsertCurrentRoom => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { room, .. } = sub_screen {
                        #[allow(clippy::collapsible_if)]
                        if room.is_valid() {
                            return match room.id {
                                Some(_id) => Action::Run(Task::perform(
                                    Room::edit(database.clone(), *room.clone()),
                                    |res| match res {
                                        Ok(_) => Message::UpsertedCurrentRoom,
                                        Err(err) => {
                                            eprintln!("{err}");
                                            Message::AddToast(Toast::error_toast(err))
                                        }
                                    },
                                )),
                                None => Action::Run(Task::perform(
                                    Room::add(database.clone(), *room.clone()),
                                    |res| match res {
                                        Ok(_) => Message::UpsertedCurrentRoom,
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
            Message::UpsertedCurrentRoom => {
                self.update(Message::FetchRooms, &database.clone(), now)
            }
            Message::DeleteCurrentRoom => {
                if let State::Ready { sub_screen, .. } = &mut self.state {
                    #[allow(clippy::collapsible_match)]
                    if let SubScreen::Upsert { room, .. } = sub_screen {
                        return Action::Run(Task::perform(
                            Room::delete(database.clone(), room.id.unwrap_or_default()),
                            |res| match res {
                                Ok(_) => Message::FetchRooms,
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
                    rooms,
                } => list_screen(pagination_state, rooms),
                SubScreen::Upsert { room, room_types } => upsert_screen(room, room_types),
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
    rooms: &'a [Room],
) -> iced::Element<'a, Message> {
    let header = list_header();
    let grid: Element<'a, Message, Theme, Renderer> = if rooms.is_empty() {
        container(text(fl!("no-rooms")).size(TITLE_TEXT_SIZE))
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
                text(fl!("room-type"))
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
            rooms.len(),
        );

        let mut grid = Column::new()
            .push(title_row)
            .spacing(GLOBAL_SPACING)
            .width(Length::Shrink);

        for room in &rooms[start_index..end_index] {
            let row = Row::new()
                .push(
                    text(&room.name)
                        .size(TEXT_SIZE)
                        .width(300.)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center),
                )
                .push(
                    text(&*room.room_type_name)
                        .size(TEXT_SIZE)
                        .width(300.)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Center),
                )
                .push(
                    row![
                        Space::new(Length::Fill, Length::Shrink),
                        button(text(fl!("edit")).size(TEXT_SIZE).align_y(Alignment::Center))
                            .on_press(Message::AskEditRoom(room.id.unwrap()))
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
        text(fl!("rooms")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        add_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}

// UPSERT SCREEN

fn upsert_screen<'a>(room: &'a Room, room_types: &'a [RoomType]) -> iced::Element<'a, Message> {
    let header = upsert_header(room);

    // Name
    let name_label = text(format!("{}*", fl!("name"))).width(Length::Fill);
    let name_input = text_input(fl!("name").as_str(), &room.name)
        .on_input(|c| Message::TextInputUpdate(c, RoomTextInputFields::Name))
        .size(TEXT_SIZE)
        .width(Length::Fill);

    let room_type_label = text(fl!("room-type")).width(Length::Fill);
    let selected = room_types.iter().find(|rt| rt.id == room.room_type_id);
    let room_type_selector = pick_list(room_types, selected, |room_type| {
        Message::UpdatedSelectedRoomTypeId(room_type.id.unwrap_or_default())
    })
    .width(Length::Fill);

    // Submit
    let submit_button_text = if room.id.is_some() {
        text(fl!("edit"))
    } else {
        text(fl!("add"))
    };
    let submit_button = button(submit_button_text.center().size(TEXT_SIZE))
        .on_press_maybe(room.is_valid().then_some(Message::UpsertCurrentRoom))
        .width(Length::Fill);

    // Input Columns
    let name_input_column = column![name_label, name_input].width(850.).spacing(1.);
    let room_type_column = column![room_type_label, room_type_selector]
        .width(850.)
        .spacing(1.);

    let form_column = Column::new()
        .push(name_input_column)
        .push(room_type_column)
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

fn upsert_header<'a>(room: &'a Room) -> iced::Element<'a, Message> {
    let back_button = button(text(fl!("back")).center())
        .on_press(Message::Back)
        .height(GLOBAL_BUTTON_HEIGHT);

    let delete_button = button(text(fl!("delete")).center())
        .style(button::danger)
        .on_press_maybe(room.id.map(|_| Message::DeleteCurrentRoom))
        .height(GLOBAL_BUTTON_HEIGHT);

    row![
        back_button,
        text(fl!("rooms")).size(TITLE_TEXT_SIZE),
        Space::new(Length::Fill, Length::Shrink),
        delete_button
    ]
    .align_y(Alignment::Center)
    .spacing(GLOBAL_SPACING)
    .padding(3.)
    .into()
}
