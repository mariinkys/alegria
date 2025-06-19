// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use iced::time::Instant;
use iced::widget::{button, center, column, container, row};
use iced::{Alignment, Length, Subscription};
use iced::{Task, widget::text};
use sqlx::{PgPool, Pool, Postgres};

use crate::alegria::screen::{self, Screen, bar, hotel, management};
use crate::alegria::widgets::toast::{self, Toast};
use crate::fl;

pub struct Alegria {
    toasts: Vec<Toast>,
    state: State,
    now: Instant,
}

enum State {
    Loading,
    Ready {
        database: Arc<Pool<Postgres>>,
        screen: Screen,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    DatabaseLoaded(Result<Arc<PgPool>, String>),

    Bar(bar::Message),
    Hotel(hotel::Message),
    Management(management::Message),

    OpenBar,
    OpenHotel,
    OpenManagement,

    AddToast(Toast),
    CloseToast(usize),
}

impl Alegria {
    pub fn new(migrate: bool) -> (Self, Task<Message>) {
        (
            Self {
                toasts: Vec::new(),
                state: State::Loading,
                now: Instant::now(),
            },
            Task::perform(
                async move { alegria_core::database::init_database(migrate).await },
                Message::DatabaseLoaded,
            ),
        )
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let content = match &self.state {
            State::Loading => center(text("Loading...")).into(),
            State::Ready { screen, .. } => match screen {
                Screen::Welcome => self.welcome_view(),
                Screen::Bar(bar) => bar.view(self.now).map(Message::Bar),
                Screen::Hotel(hotel) => hotel.view(self.now).map(Message::Hotel),
                Screen::Management(management) => {
                    management.view(self.now).map(Message::Management)
                }
            },
        };

        toast::Manager::new(content, &self.toasts, Message::CloseToast).into()
    }

    pub fn update(&mut self, message: Message, now: Instant) -> Task<Message> {
        self.now = now;

        match message {
            Message::DatabaseLoaded(db_res) => match db_res {
                Ok(pool) => {
                    self.state = State::Ready {
                        database: pool,
                        screen: Screen::Welcome,
                    }
                }
                Err(err) => {
                    eprintln!("Database init failed: {err}");
                    std::process::exit(1);
                }
            },

            Message::Bar(message) => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let Screen::Bar(bar) = screen else {
                    return Task::none();
                };

                return match bar.update(message, database, self.now) {
                    bar::Action::None => Task::none(),
                    bar::Action::Run(task) => task.map(Message::Bar),
                    bar::Action::Back => {
                        *screen = Screen::Welcome;
                        return Task::none();
                    }
                    bar::Action::AddToast(toast) => {
                        return self.update(Message::AddToast(toast), now);
                    }
                };
            }
            Message::OpenBar => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let (bar, task) = screen::Bar::new(database);
                *screen = Screen::Bar(bar);
                return task.map(Message::Bar);
            }

            Message::Hotel(message) => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let Screen::Hotel(hotel) = screen else {
                    return Task::none();
                };

                return match hotel.update(message, database, self.now) {
                    hotel::Action::None => Task::none(),
                    hotel::Action::Run(task) => task.map(Message::Hotel),
                    hotel::Action::Back => {
                        *screen = Screen::Welcome;
                        return Task::none();
                    }
                    hotel::Action::AddToast(toast) => {
                        return self.update(Message::AddToast(toast), now);
                    }
                };
            }
            Message::OpenHotel => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let (hotel, task) = screen::Hotel::new(database);
                *screen = Screen::Hotel(hotel);
                return task.map(Message::Hotel);
            }

            Message::Management(message) => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let Screen::Management(management) = screen else {
                    return Task::none();
                };

                return match management.update(message, database, self.now) {
                    management::Action::None => Task::none(),
                    management::Action::Run(task) => task.map(Message::Management),
                    management::Action::Back => {
                        *screen = Screen::Welcome;
                        return Task::none();
                    }
                    management::Action::AddToast(toast) => {
                        return self.update(Message::AddToast(toast), now);
                    }
                };
            }
            Message::OpenManagement => {
                let State::Ready {
                    screen, database, ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let (management, task) = screen::Management::new(database);
                *screen = Screen::Management(management);
                return task.map(Message::Management);
            }

            Message::AddToast(toast) => {
                self.toasts.push(toast);
            }
            Message::CloseToast(index) => {
                self.toasts.remove(index);
            }
        }

        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let State::Ready { screen, .. } = &self.state else {
            return Subscription::none();
        };

        match screen {
            Screen::Welcome => Subscription::none(),
            Screen::Bar(bar) => bar.subscription(self.now).map(Message::Bar),
            Screen::Hotel(hotel) => hotel.subscription(self.now).map(Message::Hotel),
            Screen::Management(management) => {
                management.subscription(self.now).map(Message::Management)
            }
        }
    }

    pub fn theme(&self) -> iced::Theme {
        iced::Theme::GruvboxLight
    }

    fn welcome_view(&self) -> iced::Element<'_, Message> {
        let buttons_row = row![
            button(text(fl!("bar")).center())
                .width(100.)
                .height(100.)
                .on_press(Message::OpenBar),
            button(text(fl!("hotel")).center())
                .width(100.)
                .height(100.)
                .on_press(Message::OpenHotel),
            button(text(fl!("management")).center())
                .width(100.)
                .height(100.)
                .on_press(Message::OpenManagement)
        ]
        .spacing(5.)
        .height(Length::Shrink);

        let centered_buttons = container(buttons_row).center(Length::Fill);

        let app_text = text("dev.mariinkys.Alegría dev-0.1.0")
            .align_x(Alignment::End)
            .width(Length::Fill);

        let content = column![centered_buttons, app_text]
            .width(Length::Fill)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
