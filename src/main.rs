// SPDX-License-Identifier: GPL-3.0-only

use app::IcedAlegria;
use iced::{Task, window::Settings};

mod app;
mod i18n;

fn main() -> Result<(), iced::Error> {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    iced::application("Battery Status", IcedAlegria::update, IcedAlegria::view)
        .window(Settings {
            position: iced::window::Position::Centered,
            resizable: true,
            ..Default::default()
        })
        .run_with(|| (IcedAlegria::new(), Task::none()))
}
