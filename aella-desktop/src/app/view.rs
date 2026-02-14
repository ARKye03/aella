use crate::app::types::{Message, State, ViewMode};
use iced::widget::{
    button, column, container, markdown, row, scrollable, svg, text, text_editor, text_input,
    tooltip,
};
use iced::{Center, Color, Element, Fill};

const PLUS_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/plus.svg");

pub(crate) fn view(state: &State) -> Element<'_, Message> {
    let sidebar = build_sidebar(state);
    let editor_area = build_editor_area(state);

    let content = row![sidebar, editor_area].spacing(0);

    container(content).width(Fill).height(Fill).into()
}

fn build_sidebar(state: &State) -> Element<'_, Message> {
    let sidebar_width = if state.sidebar_collapsed { 60 } else { 300 };

    let toggle_icon = if state.sidebar_collapsed {
        "☰"
    } else {
        "←"
    };
    let toggle_button = button(text(toggle_icon).size(20).align_x(Center))
        .on_press(Message::ToggleSidebar)
        .width(Fill)
        .padding(12);

    if state.sidebar_collapsed {
        let plus_icon = svg(PLUS_ICON_PATH)
            .width(20)
            .height(20)
            .style(|_theme, _status| svg::Style {
                color: Some(Color::WHITE),
            });

        let new_button = button(container(plus_icon).center_x(Fill).center_y(Fill))
            .on_press(Message::NewConversation)
            .width(Fill)
            .height(52)
            .padding(0);

        let source_conversations = if state.selected_in_trash {
            &state.trashed_conversations
        } else {
            &state.conversations
        };

        let compact_list = scrollable(
            column(
                source_conversations
                    .iter()
                    .map(|conv| {
                        let badge = compact_title_badge(&conv.title);
                        let compact_button = button(
                            container(text(badge).size(18).align_x(Center))
                                .center_x(Fill)
                                .center_y(Fill),
                        )
                        .on_press(Message::ConversationSelected(conv.id))
                        .style(if state.selected_conversation == Some(conv.id) {
                            button::primary
                        } else {
                            button::secondary
                        })
                        .width(Fill)
                        .height(52)
                        .padding(0);

                        tooltip(
                            compact_button,
                            text(conv.title.clone()),
                            tooltip::Position::Right,
                        )
                        .padding(8)
                        .style(tooltip_container_style)
                        .into()
                    })
                    .collect::<Vec<Element<'_, Message>>>(),
            )
            .spacing(8),
        );

        let toggle_with_tooltip = tooltip(
            toggle_button,
            text("Toggle Sidebar View"),
            tooltip::Position::Right,
        )
        .padding(8)
        .style(tooltip_container_style);
        let add_with_tooltip = tooltip(
            new_button,
            text("Add new conversation"),
            tooltip::Position::Right,
        )
        .padding(8)
        .style(tooltip_container_style);

        let sidebar_content =
            column![toggle_with_tooltip, add_with_tooltip, compact_list].spacing(8);

        return container(sidebar_content)
            .width(sidebar_width)
            .height(Fill)
            .padding(8)
            .into();
    }

    let search = text_input("Search conversations...", &state.search_query)
        .on_input(Message::SearchChanged)
        .padding(12);

    let plus_icon = svg(PLUS_ICON_PATH)
        .width(20)
        .height(20)
        .style(|_theme, _status| svg::Style {
            color: Some(Color::WHITE),
        });

    let new_button = button(
        row![plus_icon, text("New Conversation").size(14)]
            .spacing(8)
            .align_y(Center),
    )
    .on_press(Message::NewConversation)
    .width(Fill)
    .padding([8, 16]);

    let mode_switch = row![
        button(text("Conversations").size(13))
            .on_press(Message::ShowConversations)
            .style(if state.selected_in_trash {
                button::secondary
            } else {
                button::primary
            }),
        button(text("Trash").size(13))
            .on_press(Message::ShowTrash)
            .style(if state.selected_in_trash {
                button::primary
            } else {
                button::secondary
            }),
    ]
    .spacing(8);

    let source_conversations = if state.selected_in_trash {
        &state.trashed_conversations
    } else {
        &state.conversations
    };

    let filtered_conversations: Vec<_> = source_conversations
        .iter()
        .filter(|conv| {
            state.search_query.is_empty()
                || conv
                    .title
                    .to_lowercase()
                    .contains(&state.search_query.to_lowercase())
                || conv
                    .preview
                    .to_lowercase()
                    .contains(&state.search_query.to_lowercase())
        })
        .collect();

    let conversation_list = scrollable(
        column(
            filtered_conversations
                .iter()
                .map(|conv| {
                    let is_selected = state.selected_conversation == Some(conv.id);
                    let open_button = button(
                        column![
                            text(&conv.title),
                            text(&conv.preview).size(12).color([0.6, 0.6, 0.6]),
                        ]
                        .spacing(4)
                        .padding([12, 12]),
                    )
                    .on_press(Message::ConversationSelected(conv.id))
                    .style(if is_selected {
                        button::primary
                    } else {
                        button::secondary
                    })
                    .width(Fill);

                    let actions: Element<'_, Message> = if state.selected_in_trash {
                        row![
                            button(text("Restore").size(11))
                                .on_press(Message::RestoreConversation(conv.id))
                                .padding([8, 10])
                                .style(button::primary),
                            button(text("Delete").size(11))
                                .on_press(Message::DeleteConversationPermanently(conv.id))
                                .padding([8, 10])
                                .style(button::danger),
                        ]
                        .spacing(6)
                        .into()
                    } else {
                        button(text("Trash").size(11))
                            .on_press(Message::MoveConversationToTrash(conv.id))
                            .padding([8, 10])
                            .into()
                    };

                    Element::from(
                        column![open_button, container(actions).padding([0, 4])]
                            .spacing(6)
                            .width(Fill),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .spacing(4)
        .padding([8, 12]),
    );

    let sidebar_content = if state.selected_in_trash {
        column![toggle_button, mode_switch, search, conversation_list].spacing(8)
    } else {
        column![
            toggle_button,
            mode_switch,
            search,
            new_button,
            conversation_list
        ]
        .spacing(8)
    };

    container(sidebar_content)
        .width(sidebar_width)
        .height(Fill)
        .padding(8)
        .into()
}

fn build_editor_area(state: &State) -> Element<'_, Message> {
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
        .align_y(Center);

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
                    .style(|theme: &iced::Theme| {
                        let palette = theme.palette();
                        container::Style {
                            background: Some(iced::Background::Color(
                                palette.background.scale_alpha(0.3),
                            )),
                            border: iced::Border {
                                width: 1.0,
                                color: palette.background.scale_alpha(0.5),
                                radius: 0.0.into(),
                            },
                            ..Default::default()
                        }
                    })
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
    .align_y(Center);

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
            .style(|theme: &iced::Theme| {
                let palette = theme.palette();
                container::Style {
                    background: Some(iced::Background::Color(
                        palette.background.scale_alpha(0.35),
                    )),
                    border: iced::Border {
                        width: 0.0,
                        color: palette.background,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            }),
        container(mode_buttons)
            .width(Fill)
            .style(|theme: &iced::Theme| {
                let palette = theme.palette();
                container::Style {
                    background: Some(iced::Background::Color(palette.background.scale_alpha(0.5))),
                    border: iced::Border {
                        width: 0.0,
                        color: palette.background,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            }),
        container(editor_content).height(Fill),
        container(suggestions_panel)
            .height(panel_height)
            .style(|theme: &iced::Theme| {
                let palette = theme.palette();
                container::Style {
                    background: Some(iced::Background::Color(palette.background.scale_alpha(0.3))),
                    border: iced::Border {
                        width: 1.0,
                        color: palette.background.scale_alpha(0.5),
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                }
            }),
        cursor_indicator,
    ]
    .spacing(0);

    container(full_editor_area).width(Fill).height(Fill).into()
}

fn compact_title_badge(title: &str) -> String {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return String::from("--");
    }

    trimmed
        .chars()
        .take(2)
        .collect::<String>()
        .to_ascii_uppercase()
}

fn tooltip_container_style(theme: &iced::Theme) -> container::Style {
    let palette = theme.palette();
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            0.08, 0.09, 0.12, 0.97,
        ))),
        text_color: Some(Color::WHITE),
        border: iced::Border {
            width: 1.0,
            color: palette.background.scale_alpha(0.75),
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}
