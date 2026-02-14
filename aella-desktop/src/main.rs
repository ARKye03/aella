mod db;

use db::{ConversationData, Database};
use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use iced::widget::{
    button, column, container, markdown, row, scrollable, svg, text, text_editor, text_input,
};
use iced::{Center, Color, Element, Fill, Task};
use std::sync::Arc;

fn main() -> iced::Result {
    iced::application(
        || (State::default(), initialize_database_task()),
        update,
        view,
    )
    .title(app_title)
    .run()
}

fn app_title(_state: &State) -> String {
    String::from("Aella")
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewMode {
    RawCode,
    BothViews,
    RenderedView,
}

struct State {
    search_query: String,
    conversations: Vec<Conversation>,
    selected_conversation: Option<i64>,
    editor_content: text_editor::Content,
    sidebar_collapsed: bool,
    linter: LintGroup,
    grammar_lints: Vec<harper_core::linting::Lint>,
    view_mode: ViewMode,
    markdown_items: Vec<markdown::Item>,
    cursor_position: (usize, usize), // (line, column)
    errors_panel_collapsed: bool,
    database: Option<Database>,
    dict: Arc<FstDictionary>, // Reuse dictionary instead of creating on every keystroke
    last_checked_text: String, // Track last text to avoid redundant grammar checks
}

impl Default for State {
    fn default() -> Self {
        let dict = FstDictionary::curated();
        let linter = LintGroup::new_curated(dict.clone(), Dialect::American);

        let initial_text = "# Welcome to Aella\n\n\
                Start typing to check your grammar in real-time.\n\n\
                This is a markdown editor with built-in grammar checking powered by Harper.";

        let markdown_items: Vec<markdown::Item> = markdown::parse(initial_text).collect();

        Self {
            search_query: String::new(),
            conversations: Vec::new(), // Start with empty list, will load from DB
            selected_conversation: None,
            editor_content: text_editor::Content::with_text(initial_text),
            sidebar_collapsed: false,
            linter,
            grammar_lints: Vec::new(),
            view_mode: ViewMode::BothViews,
            markdown_items,
            cursor_position: (1, 1),
            errors_panel_collapsed: false,
            database: None,
            dict, // Store dictionary for reuse
            last_checked_text: initial_text.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct Conversation {
    id: i64,
    title: String,
    preview: String,
}

#[derive(Debug, Clone)]
enum Message {
    SearchChanged(String),
    ConversationSelected(i64),
    NewConversation,
    EditorAction(text_editor::Action),
    ToggleSidebar,
    SetViewMode(ViewMode),
    MarkdownLinkClicked(markdown::Uri),
    ToggleErrorsPanel,
    DatabaseInitialized(Database),
    ConversationsLoaded(Vec<ConversationData>),
    ConversationContentLoaded(String), // Load just content, not entire list
    GrammarChecked {
        content: String,
        lints: Vec<harper_core::linting::Lint>,
    },
    ConversationSaved,
    Noop,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::SearchChanged(query) => {
            state.search_query = query;
        }
        Message::ConversationSelected(selected_id) => {
            // Save current conversation before switching
            if let (Some(database), Some(current_id)) =
                (&state.database, state.selected_conversation)
            {
                if let Some(conversation) = state
                    .conversations
                    .iter()
                    .find(|conversation| conversation.id == current_id)
                {
                    let db_clone = database.clone();
                    let id = conversation.id;
                    let title = conversation.title.clone();
                    let content = state.editor_content.text();

                    let save_task = Task::perform(
                        async move {
                            db_clone
                                .update_conversation(id, &title, &content)
                                .await
                                .ok();
                        },
                        |_| Message::ConversationSaved,
                    );

                    // Load the selected conversation content
                    state.selected_conversation = Some(selected_id);
                    if state
                        .conversations
                        .iter()
                        .any(|conv| conv.id == selected_id)
                    {
                        let db_clone2 = database.clone();

                        return Task::batch(vec![
                            save_task,
                            Task::perform(
                                async move {
                                    db_clone2
                                        .get_conversation(selected_id)
                                        .await
                                        .ok()
                                        .flatten()
                                        .map(|conv_data| conv_data.content)
                                        .unwrap_or_default()
                                },
                                Message::ConversationContentLoaded,
                            ),
                        ]);
                    }
                    return save_task;
                }
            }

            state.selected_conversation = Some(selected_id);
            if let Some(database) = &state.database {
                let db_clone = database.clone();
                return Task::perform(
                    async move {
                        db_clone
                            .get_conversation(selected_id)
                            .await
                            .ok()
                            .flatten()
                            .map(|conv_data| conv_data.content)
                            .unwrap_or_default()
                    },
                    Message::ConversationContentLoaded,
                );
            }
        }
        Message::NewConversation => {
            if let Some(database) = &state.database {
                let db_clone = database.clone();
                let title = String::from("New Conversation");
                let content = String::new();

                return Task::perform(
                    async move {
                        let _new_id = db_clone
                            .create_conversation(&title, &content)
                            .await
                            .unwrap_or(0);
                        // Reload all conversations after creating
                        db_clone.get_all_conversations().await.unwrap_or_default()
                    },
                    Message::ConversationsLoaded,
                );
            } else {
                // Fallback if database not ready
                let new_id = state.conversations.len() + 1;
                state.conversations.insert(
                    0,
                    Conversation {
                        id: new_id as i64,
                        title: format!("New Conversation {}", new_id),
                        preview: String::from("Start writing..."),
                    },
                );
                state.selected_conversation = Some(new_id as i64);
                state.editor_content = text_editor::Content::new();
            }
        }
        Message::EditorAction(action) => {
            state.editor_content.perform(action);

            let text = state.editor_content.text();

            // Only run grammar checking if text actually changed (not just cursor movement)
            if text != state.last_checked_text {
                let document = Document::new_markdown_default(&text, &state.dict);
                state.grammar_lints = state.linter.lint(&document);

                // Re-parse markdown
                state.markdown_items = markdown::parse(&text).collect();

                state.last_checked_text = text;
            }

            // Always update cursor position (lightweight operation)
            state.cursor_position = calculate_cursor_position(&state.editor_content);
        }
        Message::ToggleSidebar => {
            state.sidebar_collapsed = !state.sidebar_collapsed;
        }
        Message::SetViewMode(mode) => {
            state.view_mode = mode;
        }
        Message::MarkdownLinkClicked(_url) => {
            // Handle markdown link clicks if needed
        }
        Message::ToggleErrorsPanel => {
            state.errors_panel_collapsed = !state.errors_panel_collapsed;
        }
        Message::DatabaseInitialized(db) => {
            state.database = Some(db);
            // Load conversations from database
            if let Some(database) = &state.database {
                let db_clone = database.clone();
                return Task::perform(
                    async move { db_clone.get_all_conversations().await.unwrap_or_default() },
                    Message::ConversationsLoaded,
                );
            }
        }
        Message::ConversationsLoaded(db_conversations) => {
            // Convert database conversations to UI conversations
            state.conversations = db_conversations
                .iter()
                .map(|conv| Conversation {
                    id: conv.id,
                    title: conv.title.clone(),
                    preview: conv.content.chars().take(50).collect::<String>() + "...",
                })
                .collect();

            // Load the most recently edited conversation (first due to ORDER BY updated_at DESC)
            if !state.conversations.is_empty() {
                state.selected_conversation = Some(state.conversations[0].id);
                if let Some(conv) = db_conversations.first() {
                    state.editor_content = text_editor::Content::with_text(&conv.content);
                    state.markdown_items = markdown::parse(&conv.content).collect();
                    state.last_checked_text = conv.content.clone();
                    return run_grammar_check_task(state.dict.clone(), conv.content.clone());
                }
            }
        }
        Message::ConversationContentLoaded(content) => {
            // Update editor with loaded content without touching conversations list
            state.editor_content = text_editor::Content::with_text(&content);
            state.markdown_items = markdown::parse(&content).collect();
            state.last_checked_text = content.clone();
            return run_grammar_check_task(state.dict.clone(), content);
        }
        Message::GrammarChecked { content, lints } => {
            // Drop stale lint results that completed after the user changed text.
            if state.last_checked_text == content {
                state.grammar_lints = lints;
            }
        }
        Message::Noop => {}
        Message::ConversationSaved => {
            // Conversation saved successfully
        }
    }
    Task::none()
}

fn initialize_database_task() -> Task<Message> {
    Task::perform(async { Database::new().await.ok() }, |db_opt| {
        if let Some(db) = db_opt {
            Message::DatabaseInitialized(db)
        } else {
            Message::Noop
        }
    })
}

fn run_grammar_check_task(dict: Arc<FstDictionary>, content: String) -> Task<Message> {
    Task::perform(
        async move {
            let dict_for_doc = dict.clone();
            let mut linter = LintGroup::new_curated(dict, Dialect::American);
            let document = Document::new_markdown_default(&content, &dict_for_doc);
            let lints = linter.lint(&document);
            (content, lints)
        },
        |(content, lints)| Message::GrammarChecked { content, lints },
    )
}

fn calculate_cursor_position(content: &text_editor::Content) -> (usize, usize) {
    let cursor = content.cursor();
    // Convert to 1-indexed (line + 1, column + 1) for display
    (cursor.position.line + 1, cursor.position.column + 1)
}

fn view(state: &State) -> Element<'_, Message> {
    let sidebar = build_sidebar(state);
    let editor_area = build_editor_area(state);

    let content = row![sidebar, editor_area].spacing(0);

    container(content).width(Fill).height(Fill).into()
}

fn build_sidebar(state: &State) -> Element<'_, Message> {
    let sidebar_width = if state.sidebar_collapsed { 60 } else { 300 };

    // Toggle button
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
        // Collapsed view - just toggle button and new conversation icon
        let plus_icon = svg(svg::Handle::from_path("assets/plus.svg"))
            .width(24)
            .height(24)
            .style(|_theme, _status| svg::Style {
                color: Some(Color::WHITE),
            });

        let new_button = button(container(plus_icon).center(Fill))
            .on_press(Message::NewConversation)
            .width(Fill)
            .padding(12);

        let sidebar_content = column![toggle_button, new_button].spacing(8);

        return container(sidebar_content)
            .width(sidebar_width)
            .height(Fill)
            .padding(8)
            .into();
    }

    // Expanded view
    let search = text_input("Search conversations...", &state.search_query)
        .on_input(Message::SearchChanged)
        .padding(12);

    let plus_icon = svg(svg::Handle::from_path("assets/plus.svg"))
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

    let filtered_conversations: Vec<&Conversation> = state
        .conversations
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
                    let conv_button = button(
                        column![
                            text(&conv.title),
                            text(&conv.preview).size(12).color([0.6, 0.6, 0.6]),
                        ]
                        .spacing(4)
                        .padding([12, 16]),
                    )
                    .on_press(Message::ConversationSelected(conv.id))
                    .width(Fill);

                    Element::from(conv_button)
                })
                .collect::<Vec<_>>(),
        )
        .spacing(4)
        .padding([8, 12]),
    );

    let sidebar_content = column![toggle_button, search, new_button, conversation_list].spacing(8);

    container(sidebar_content)
        .width(sidebar_width)
        .height(Fill)
        .padding(8)
        .into()
}

fn build_editor_area(state: &State) -> Element<'_, Message> {
    // View mode buttons
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

    // Editor content based on view mode
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

    // Grammar suggestions panel
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
                .take(10) // Show max 10 suggestions
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

    // Cursor position indicator
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

    // Combine mode buttons, editor content, and suggestions in a column
    let full_editor_area = column![
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
