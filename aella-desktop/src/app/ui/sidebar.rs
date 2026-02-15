use crate::app::sidebar_search_input_id;
use crate::app::types::{Message, State};
use crate::app::ui::styles::tooltip_container_style;
use iced::widget::{button, column, container, row, scrollable, svg, text, text_input, tooltip};
use iced::{Center, Color, Element, Fill};

const PLUS_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/plus.svg");
const SIDEBAR_WIDTH_COLLAPSED: u32 = 60;
const SIDEBAR_WIDTH_EXPANDED: u32 = 300;

pub(crate) fn build_sidebar(state: &State) -> Element<'_, Message> {
    let sidebar_width = if state.sidebar_collapsed {
        SIDEBAR_WIDTH_COLLAPSED
    } else {
        SIDEBAR_WIDTH_EXPANDED
    };

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
        .id(sidebar_search_input_id())
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
    let search_query = state.search_query.trim().to_lowercase();

    let filtered_conversations: Vec<_> = source_conversations
        .iter()
        .filter(|conv| {
            search_query.is_empty()
                || conv.title.to_lowercase().contains(&search_query)
                || conv.preview.to_lowercase().contains(&search_query)
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
