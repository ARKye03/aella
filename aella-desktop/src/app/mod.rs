pub(crate) mod types;
mod ui;
mod update;
mod view;

use crate::app::types::Message;
use iced::Subscription;
use iced::event;
use iced::keyboard;
use iced::time;
use iced::widget::Id;
use std::time::Duration;

pub(crate) use types::State;
pub(crate) use update::{initialize_database_task, update};
pub(crate) use view::view;

pub(crate) const ANIMATION_TICK_INTERVAL: Duration = Duration::from_millis(16);
pub(crate) const AUTOSAVE_DEBOUNCE: Duration = Duration::from_millis(1200);
pub(crate) const GRAMMAR_CHECK_DEBOUNCE: Duration = Duration::from_millis(350);

pub(crate) fn app_title(_state: &State) -> String {
    String::from("Aella")
}

pub(crate) fn subscription(state: &State) -> Subscription<Message> {
    let events = event::listen_with(|event, _status, _window| match event {
        iced::Event::Keyboard(keyboard_event) => Some(Message::KeyboardEvent(keyboard_event)),
        _ => None,
    });

    if state.shortcuts_help_animation.is_animating(state.now) {
        return Subscription::batch([
            events,
            time::every(ANIMATION_TICK_INTERVAL).map(Message::Tick),
        ]);
    }

    events
}

pub(crate) fn primary_shortcut_modifier_pressed(modifiers: keyboard::Modifiers) -> bool {
    modifiers.logo() || modifiers.control()
}

pub(crate) fn sidebar_search_input_id() -> Id {
    Id::new("sidebar-search-input")
}
