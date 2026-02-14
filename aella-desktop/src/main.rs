mod app;
mod db;

use app::{State, app_title, initialize_database_task, update, view};

fn main() -> iced::Result {
    iced::application(
        || (State::default(), initialize_database_task()),
        update,
        view,
    )
    .title(app_title)
    .run()
}
