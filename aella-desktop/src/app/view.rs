use crate::app::types::{Message, State};
use crate::app::ui::styles::{modal_backdrop_style, modal_card_style};
use crate::app::ui::{editor::build_editor_area, sidebar::build_sidebar};
use iced::widget::{button, column, container, opaque, row, stack, text};
use iced::{Element, Fill};

pub(crate) fn view(state: &State) -> Element<'_, Message> {
    let sidebar = build_sidebar(state);
    let editor_area = build_editor_area(state);
    let content = row![sidebar, editor_area].spacing(0);
    let base = container(content).width(Fill).height(Fill);

    if !state.shortcuts_help_open {
        return base.into();
    }

    let modifier = if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    };

    let shortcuts = [
        (format!("{modifier} + F"), "Expand sidebar + focus search"),
        (format!("{modifier} + 1"), "Raw mode"),
        (format!("{modifier} + 2"), "Split mode"),
        (format!("{modifier} + 3"), "Rendered mode"),
        (format!("{modifier} + B"), "Toggle sidebar"),
        (format!("{modifier} + Shift + B"), "Toggle issues panel"),
        (format!("{modifier} + N"), "New conversation"),
        (format!("{modifier} + ?"), "Show/hide this help"),
        ("Esc".to_string(), "Close shortcut help"),
    ];

    let shortcut_list = column(
        shortcuts
            .iter()
            .map(|(shortcut, action)| {
                row![
                    container(text(shortcut.clone()).size(14)).width(160),
                    text(action.to_string()).size(14),
                ]
                .spacing(12)
                .into()
            })
            .collect::<Vec<Element<'_, Message>>>(),
    )
    .spacing(8);

    let modal = container(
        column![
            row![
                text("Keyboard Shortcuts").size(24),
                button("Close")
                    .on_press(Message::ToggleShortcutsHelp)
                    .padding([6, 10])
            ]
            .spacing(16),
            shortcut_list
        ]
        .spacing(16),
    )
    .max_width(560)
    .padding(20)
    .style(modal_card_style);

    let overlay = container(container(modal).center_x(Fill).center_y(Fill))
        .width(Fill)
        .height(Fill)
        .style(modal_backdrop_style);

    stack(vec![base.into(), opaque(overlay)])
        .width(Fill)
        .height(Fill)
        .into()
}
