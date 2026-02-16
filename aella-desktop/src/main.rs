mod app;

use app::subscription;
use app::{State, app_title, initialize_database_task, update, view};
use iced::Font;
use iced::window;

const GEIST_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/Geist-Regular.otf"
));
const GEIST_FONT: Font = Font::with_name("Geist");

fn main() -> iced::Result {
    let window_settings = window::Settings {
        icon: app_icon(),
        ..window::Settings::default()
    };

    iced::application(
        || (State::default(), initialize_database_task()),
        update,
        view,
    )
    .font(GEIST_FONT_BYTES)
    .default_font(GEIST_FONT)
    .title(app_title)
    .subscription(subscription)
    .window(window_settings)
    .run()
}

fn app_icon() -> Option<window::Icon> {
    #[cfg(target_os = "windows")]
    {
        let bytes = include_bytes!("../assets/aellaWindowsIcon.ico");
        window::icon::from_file_data(bytes, None).ok()
    }

    #[cfg(not(target_os = "windows"))]
    {
        let bytes = include_bytes!("../assets/aella.icns");
        window::icon::from_file_data(bytes, None).ok()
    }
}
