pub(crate) mod types;
mod ui;
mod update;
mod view;

pub(crate) use types::State;
pub(crate) use update::{initialize_database_task, update};
pub(crate) use view::view;

pub(crate) fn app_title(_state: &State) -> String {
    String::from("Aella")
}
