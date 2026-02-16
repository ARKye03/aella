use crate::app::types::{Message, State};
use crate::app::ui::styles::{
    app_shell_style, modal_backdrop_style_with_alpha, modal_card_style_with_alpha,
};
use crate::app::ui::{editor::build_editor_area, sidebar::build_sidebar};
use iced::widget::{column, container, mouse_area, opaque, row, stack, text};
use iced::{Element, Fill};

pub(crate) fn view(state: &State) -> Element<'_, Message> {
    let sidebar = build_sidebar(state);
    let editor_area = build_editor_area(state);
    let content = row![sidebar, editor_area].spacing(14).padding(14);
    let base = container(content)
        .width(Fill)
        .height(Fill)
        .style(app_shell_style);

    let transition = state
        .shortcuts_help_animation
        .interpolate(0.0_f32, 1.0_f32, state.now)
        .clamp(0.0, 1.0);

    if transition <= 0.001 {
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
        (format!("{modifier} + K"), "Show/hide this help"),
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

    let modal_width = (500.0 + (transition * 60.0)) as u32;
    let modal_padding = (14.0 + (transition * 6.0)) as u16;
    let modal_y_offset = ((1.0 - transition) * 22.0) as u16;

    let modal = container(column![text("Keyboard Shortcuts").size(24), shortcut_list].spacing(16))
        .max_width(modal_width)
        .padding(modal_padding)
        .style(modal_card_style_with_alpha(transition));

    let centered_modal = container(mouse_area(modal).on_press(Message::Noop))
        .center_x(Fill)
        .center_y(Fill)
        .padding([modal_y_offset, 0])
        .width(Fill)
        .height(Fill);

    let overlay = mouse_area(
        container(centered_modal)
            .width(Fill)
            .height(Fill)
            .style(modal_backdrop_style_with_alpha(transition)),
    )
    .on_press(Message::CloseShortcutsHelp);

    let overlay = container(overlay).width(Fill).height(Fill);

    stack(vec![base.into(), opaque(overlay)])
        .width(Fill)
        .height(Fill)
        .into()
}
