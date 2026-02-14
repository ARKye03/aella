use crate::app::types::{Message, State, ViewMode};
use crate::app::ui::styles::{container_bg, outlined_container_bg};
use iced::widget::{
    button, column, container, markdown, row, scrollable, text, text_editor, text_input,
};
use iced::{Element, Fill};

pub(crate) fn build_editor_area(state: &State) -> Element<'_, Message> {
    let title_input = text_input("Conversation title", &state.current_title)
        .on_input(Message::TitleChanged)
        .on_submit(Message::SaveTitle)
        .padding(10)
        .width(Fill);

    let title_actions: Element<'_, Message> = if state.selected_in_trash {
        text("This conversation is in Trash").size(12).into()
    } else if let Some(id) = state.selected_conversation {
        button(text("Move to Trash").size(12))
            .on_press(Message::MoveConversationToTrash(id))
            .padding([8, 10])
            .into()
    } else {
        text("").into()
    };

    let title_bar = row![title_input, title_actions]
        .spacing(8)
        .padding([8, 16])
        .align_y(iced::Center);

    let mode_buttons = row![
        button(text("Raw Code").size(13))
            .on_press(Message::SetViewMode(ViewMode::RawCode))
            .padding([6, 12])
            .style(if state.view_mode == ViewMode::RawCode {
                button::primary
            } else {
                button::secondary
            }),
        button(text("Both Views").size(13))
            .on_press(Message::SetViewMode(ViewMode::BothViews))
            .padding([6, 12])
            .style(if state.view_mode == ViewMode::BothViews {
                button::primary
            } else {
                button::secondary
            }),
        button(text("Rendered").size(13))
            .on_press(Message::SetViewMode(ViewMode::RenderedView))
            .padding([6, 12])
            .style(if state.view_mode == ViewMode::RenderedView {
                button::primary
            } else {
                button::secondary
            }),
    ]
    .spacing(8)
    .padding([12, 16]);

    let editor_content: Element<'_, Message> = match state.view_mode {
        ViewMode::RawCode => {
            let editor = text_editor(&state.editor_content)
                .on_action(Message::EditorAction)
                .height(Fill)
                .padding(24);

            container(editor).width(Fill).height(Fill).into()
        }
        ViewMode::BothViews => {
            let editor = text_editor(&state.editor_content)
                .on_action(Message::EditorAction)
                .height(Fill)
                .padding(24);

            let rendered = scrollable(
                markdown::view(&state.markdown_items, iced::Theme::TokyoNight)
                    .map(Message::MarkdownLinkClicked),
            )
            .height(Fill);

            row![
                container(editor).width(Fill).height(Fill),
                container(rendered)
                    .width(Fill)
                    .height(Fill)
                    .padding(24)
                    .style(|theme: &iced::Theme| outlined_container_bg(theme, 0.3, 0.5))
            ]
            .spacing(0)
            .into()
        }
        ViewMode::RenderedView => {
            let rendered = scrollable(
                markdown::view(&state.markdown_items, iced::Theme::TokyoNight)
                    .map(Message::MarkdownLinkClicked),
            )
            .height(Fill);

            container(rendered)
                .width(Fill)
                .height(Fill)
                .padding(24)
                .into()
        }
    };

    let panel_height = if state.errors_panel_collapsed {
        40
    } else {
        200
    };

    let toggle_icon = if state.errors_panel_collapsed {
        "▲"
    } else {
        "▼"
    };
    let issue_count = state.grammar_lints.len();
    let status_text = if issue_count == 0 {
        "No issues found ✓".to_string()
    } else {
        format!("{} issue(s)", issue_count)
    };

    let panel_header = row![
        text(status_text).size(13).color(if issue_count == 0 {
            [0.5, 0.8, 0.5]
        } else {
            [1.0, 0.8, 0.5]
        }),
        button(text(toggle_icon).size(12))
            .on_press(Message::ToggleErrorsPanel)
            .padding([4, 8])
            .style(button::text),
    ]
    .spacing(12)
    .align_y(iced::Center);

    let suggestions_panel = if state.errors_panel_collapsed {
        container(panel_header).padding(12).width(Fill)
    } else if state.grammar_lints.is_empty() {
        container(
            column![
                panel_header,
                text("Your grammar is perfect!")
                    .size(12)
                    .color([0.7, 0.7, 0.7])
            ]
            .spacing(8),
        )
        .padding(12)
        .width(Fill)
    } else {
        let lint_list = column(
            state
                .grammar_lints
                .iter()
                .take(10)
                .map(|lint| {
                    let message = lint.message.clone();
                    let suggestion_text = if let Some(suggestion) = lint.suggestions.first() {
                        format!("→ {}", suggestion)
                    } else {
                        String::from("(no suggestion)")
                    };

                    Element::from(
                        column![
                            text(message).size(13).color([1.0, 0.7, 0.7]),
                            text(suggestion_text).size(12).color([0.7, 0.7, 0.7]),
                        ]
                        .spacing(4)
                        .padding([8, 12]),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .spacing(8);

        container(column![panel_header, scrollable(lint_list).height(Fill),].spacing(8))
            .padding(12)
            .width(Fill)
    };

    let cursor_indicator = container(
        text(format!(
            "Ln {}, Col {}",
            state.cursor_position.0, state.cursor_position.1
        ))
        .size(12)
        .color([0.6, 0.6, 0.6]),
    )
    .padding([4, 12])
    .align_x(iced::alignment::Horizontal::Right);

    let full_editor_area = column![
        container(title_bar)
            .width(Fill)
            .style(|theme: &iced::Theme| container_bg(theme, 0.35)),
        container(mode_buttons)
            .width(Fill)
            .style(|theme: &iced::Theme| container_bg(theme, 0.5)),
        container(editor_content).height(Fill),
        container(suggestions_panel)
            .height(panel_height)
            .style(|theme: &iced::Theme| outlined_container_bg(theme, 0.3, 0.5)),
        cursor_indicator,
    ]
    .spacing(0);

    container(full_editor_area).width(Fill).height(Fill).into()
}
