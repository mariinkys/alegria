// SPDX-License-Identifier: GPL-3.0-only

use app::IcedAlegria;
use iced::{Task, window::Settings};

mod alegria;
mod app;
mod i18n;

/// Unique identifier in RDNN (reverse domain name notation) format.
const APP_ID: &str = "dev.mariinkys.IcedAlegria";

fn main() -> Result<(), iced::Error> {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Tasks that will get executed on the application init
    let mut tasks = vec![];

    tasks.push(iced::Task::perform(
        async move { alegria::core::database::init_database(APP_ID).await },
        app::Message::DatabaseLoaded,
    ));

    iced::application(APP_ID, IcedAlegria::update, IcedAlegria::view)
        .window(Settings {
            position: iced::window::Position::Centered,
            resizable: true,
            ..Default::default()
        })
        .theme(IcedAlegria::theme)
        .run_with(|| (IcedAlegria::new(), Task::batch(tasks)))
}
