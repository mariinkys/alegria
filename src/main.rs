// SPDX-License-Identifier: GPL-3.0-only

use app::IcedAlegria;
use iced::{
    Size,
    window::{Settings, icon},
};

mod alegria;
mod app;
mod i18n;

#[allow(clippy::empty_line_after_doc_comments)]
/// Access glibc malloc tunables.
// #[cfg(target_env = "gnu")]
// mod malloc {
//     use std::os::raw::c_int;
//     const M_MMAP_THRESHOLD: c_int = -3;

//     unsafe extern "C" {
//         fn mallopt(param: c_int, value: c_int) -> c_int;
//     }

//     /// Prevents glibc from hoarding memory via memory fragmentation.
//     pub fn limit_mmap_threshold() {
//         unsafe {
//             mallopt(M_MMAP_THRESHOLD, 65536);
//         }
//     }
// }

fn main() -> iced::Result {
    // #[cfg(target_env = "gnu")]
    // malloc::limit_mmap_threshold();
    let args: Vec<String> = std::env::args().collect();
    let migrate = args.contains(&"-m".to_string());

    // Get the window  icon
    let icon = icon::from_file_data(
        include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg"),
        None,
    );

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    iced::application::timed(
        move || IcedAlegria::new(migrate),
        IcedAlegria::update,
        IcedAlegria::subscription,
        IcedAlegria::view,
    )
    .window(Settings {
        position: iced::window::Position::Centered,
        icon: icon.ok(),
        resizable: true,
        size: Size::new(1200., 850.),
        min_size: Some(Size::new(1200., 850.)),
        ..Default::default()
    })
    .theme(IcedAlegria::theme)
    .run()
}
