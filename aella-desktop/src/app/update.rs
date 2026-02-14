use crate::app::types::{Conversation, Message, State};
use crate::db::Database;
use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};
use iced::Task;
use iced::widget::{markdown, text_editor};
use std::cmp::Reverse;
use std::collections::HashSet;
use std::sync::Arc;

pub(crate) fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::SearchChanged(query) => {
            state.search_query = query;
        }
        Message::TitleChanged(title) => {
            state.current_title = title;
            if let Some(selected_id) = state.selected_conversation {
                if state.selected_in_trash {
                    if let Some(conv) = state
                        .trashed_conversations
                        .iter_mut()
                        .find(|conv| conv.id == selected_id)
                    {
                        conv.title = state.current_title.clone();
                    }
                } else if let Some(conv) = state
                    .conversations
                    .iter_mut()
                    .find(|conv| conv.id == selected_id)
                {
                    conv.title = state.current_title.clone();
                }
            }
        }
        Message::SaveTitle => {
            return save_current_conversation_task(state);
        }
        Message::ConversationSelected(selected_id) => {
            let save_task = save_current_conversation_task(state);
            state.selected_conversation = Some(selected_id);
            state.selected_in_trash = state
                .trashed_conversations
                .iter()
                .any(|conv| conv.id == selected_id);
            return Task::batch(vec![save_task, load_conversation_task(state, selected_id)]);
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
                    },
                    |_| Message::RefreshLists,
                );
            } else {
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
                state.current_title = format!("New Conversation {}", new_id);
                state.editor_content = text_editor::Content::new();
            }
        }
        Message::ShowConversations => {
            state.selected_in_trash = false;
            state.selected_conversation = state.conversations.first().map(|conv| conv.id);
            if let Some(id) = state.selected_conversation {
                return load_conversation_task(state, id);
            }
        }
        Message::ShowTrash => {
            state.selected_in_trash = true;
            state.selected_conversation = state.trashed_conversations.first().map(|conv| conv.id);
            if let Some(id) = state.selected_conversation {
                return load_conversation_task(state, id);
            }
        }
        Message::MoveConversationToTrash(id) => {
            if let Some(database) = &state.database {
                let db_clone = database.clone();
                if state.selected_conversation == Some(id) {
                    clear_loaded_conversation(state);
                }
                return Task::perform(
                    async move {
                        db_clone.trash_conversation(id).await.ok();
                    },
                    |_| Message::RefreshLists,
                );
            }
        }
        Message::RestoreConversation(id) => {
            if let Some(database) = &state.database {
                let db_clone = database.clone();
                return Task::perform(
                    async move {
                        db_clone.restore_conversation(id).await.ok();
                    },
                    |_| Message::RefreshLists,
                );
            }
        }
        Message::DeleteConversationPermanently(id) => {
            if let Some(database) = &state.database {
                let db_clone = database.clone();
                if state.selected_conversation == Some(id) {
                    clear_loaded_conversation(state);
                }
                return Task::perform(
                    async move {
                        db_clone.delete_conversation(id).await.ok();
                    },
                    |_| Message::RefreshLists,
                );
            }
        }
        Message::ApplyLintSuggestion(index) => {
            apply_single_suggestion(state, index);
        }
        Message::ApplyAllSuggestions => {
            apply_all_suggestions(state);
        }
        Message::EditorAction(action) => {
            state.editor_content.perform(action);

            let text = state.editor_content.text();

            if text != state.last_checked_text {
                let document = Document::new_markdown_default(&text, &state.dict);
                state.grammar_lints = state.linter.lint(&document);
                state.markdown_items = markdown::parse(&text).collect();
                state.last_checked_text = text;
            }

            state.cursor_position = calculate_cursor_position(&state.editor_content);
        }
        Message::ToggleSidebar => {
            state.sidebar_collapsed = !state.sidebar_collapsed;
        }
        Message::SetViewMode(mode) => {
            state.view_mode = mode;
        }
        Message::MarkdownLinkClicked(_url) => {}
        Message::ToggleErrorsPanel => {
            state.errors_panel_collapsed = !state.errors_panel_collapsed;
        }
        Message::DatabaseInitialized(db) => {
            state.database = Some(db);
            return refresh_lists_task(state);
        }
        Message::ConversationsLoaded(db_conversations) => {
            let had_selection = state.selected_conversation;
            state.conversations = db_conversations
                .iter()
                .map(|conv| Conversation {
                    id: conv.id,
                    title: conv.title.clone(),
                    preview: preview(&conv.content),
                })
                .collect();

            if !state.selected_in_trash {
                let next_selected = had_selection.and_then(|selected_id| {
                    state
                        .conversations
                        .iter()
                        .find(|conv| conv.id == selected_id)
                        .map(|_| selected_id)
                });
                state.selected_conversation =
                    next_selected.or_else(|| state.conversations.first().map(|conv| conv.id));
                if let Some(id) = state.selected_conversation {
                    return load_conversation_task(state, id);
                }
            }
        }
        Message::TrashedConversationsLoaded(db_conversations) => {
            let had_selection = state.selected_conversation;
            state.trashed_conversations = db_conversations
                .iter()
                .map(|conv| Conversation {
                    id: conv.id,
                    title: conv.title.clone(),
                    preview: preview(&conv.content),
                })
                .collect();

            if state.selected_in_trash {
                let next_selected = had_selection.and_then(|selected_id| {
                    state
                        .trashed_conversations
                        .iter()
                        .find(|conv| conv.id == selected_id)
                        .map(|_| selected_id)
                });
                state.selected_conversation = next_selected
                    .or_else(|| state.trashed_conversations.first().map(|conv| conv.id));
                if let Some(id) = state.selected_conversation {
                    return load_conversation_task(state, id);
                }
            }
        }
        Message::ConversationDataLoaded(conv) => {
            state.current_title = conv.title;
            state.editor_content = text_editor::Content::with_text(&conv.content);
            state.markdown_items = markdown::parse(&conv.content).collect();
            state.last_checked_text = conv.content.clone();
            return run_grammar_check_task(state.dict.clone(), conv.content);
        }
        Message::RefreshLists => {
            return refresh_lists_task(state);
        }
        Message::GrammarChecked { content, lints } => {
            if state.last_checked_text == content {
                state.grammar_lints = lints;
            }
        }
        Message::Noop => {}
        Message::ConversationSaved => {}
    }
    Task::none()
}

pub(crate) fn initialize_database_task() -> Task<Message> {
    Task::perform(async { Database::new().await.ok() }, |db_opt| {
        if let Some(db) = db_opt {
            Message::DatabaseInitialized(db)
        } else {
            Message::Noop
        }
    })
}

fn save_current_conversation_task(state: &State) -> Task<Message> {
    if let (Some(database), Some(selected_id)) = (&state.database, state.selected_conversation) {
        let db_clone = database.clone();
        let title = state.current_title.clone();
        let content = state.editor_content.text();
        return Task::perform(
            async move {
                db_clone
                    .update_conversation(selected_id, &title, &content)
                    .await
                    .ok();
            },
            |_| Message::ConversationSaved,
        );
    }
    Task::none()
}

fn load_conversation_task(state: &State, id: i64) -> Task<Message> {
    if let Some(database) = &state.database {
        let db_clone = database.clone();
        return Task::perform(
            async move { db_clone.get_conversation(id).await.ok().flatten() },
            |conv| {
                if let Some(data) = conv {
                    Message::ConversationDataLoaded(data)
                } else {
                    Message::Noop
                }
            },
        );
    }
    Task::none()
}

fn refresh_lists_task(state: &State) -> Task<Message> {
    if let Some(database) = &state.database {
        let active_db = database.clone();
        let trash_db = database.clone();
        return Task::batch(vec![
            Task::perform(
                async move { active_db.get_all_conversations().await.unwrap_or_default() },
                Message::ConversationsLoaded,
            ),
            Task::perform(
                async move {
                    trash_db
                        .get_trashed_conversations()
                        .await
                        .unwrap_or_default()
                },
                Message::TrashedConversationsLoaded,
            ),
        ]);
    }
    Task::none()
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
    (cursor.position.line + 1, cursor.position.column + 1)
}

fn preview(content: &str) -> String {
    if content.is_empty() {
        return String::from("Start writing...");
    }
    let mut text = content.chars().take(50).collect::<String>();
    if content.chars().count() > 50 {
        text.push_str("...");
    }
    text
}

fn clear_loaded_conversation(state: &mut State) {
    state.selected_conversation = None;
    state.current_title = String::from("Untitled");
    state.editor_content = text_editor::Content::new();
    state.markdown_items = Vec::new();
    state.grammar_lints = Vec::new();
    state.last_checked_text.clear();
}

fn apply_single_suggestion(state: &mut State, lint_index: usize) {
    let Some(lint) = state.grammar_lints.get(lint_index) else {
        return;
    };
    let Some(suggestion) = lint.suggestions.first() else {
        return;
    };

    let mut chars: Vec<char> = state.editor_content.text().chars().collect();
    suggestion.apply(lint.span, &mut chars);
    let updated = chars.into_iter().collect::<String>();
    set_content_and_relint(state, updated);
}

fn apply_all_suggestions(state: &mut State) {
    const MAX_PASSES: usize = 8;

    let mut content = state.editor_content.text();

    for _ in 0..MAX_PASSES {
        let document = Document::new_markdown_default(&content, &state.dict);
        let lints = state.linter.lint(&document);

        // Keep only one suggestion per exact lint span.
        let mut seen_spans = HashSet::new();
        let mut edits = Vec::new();
        for lint in &lints {
            let Some(suggestion) = lint.suggestions.first() else {
                continue;
            };
            let key = (lint.span.start, lint.span.end);
            if seen_spans.insert(key) {
                edits.push((lint.span, suggestion.clone()));
            }
        }

        if edits.is_empty() {
            break;
        }

        // Apply from end to start to keep earlier spans stable within this pass.
        edits.sort_by_key(|(span, _)| Reverse(span.start));
        let mut chars: Vec<char> = content.chars().collect();
        for (span, suggestion) in edits {
            suggestion.apply(span, &mut chars);
        }

        let updated = chars.into_iter().collect::<String>();
        if updated == content {
            break;
        }

        content = updated;
    }

    set_content_and_relint(state, content);
}

fn set_content_and_relint(state: &mut State, content: String) {
    state.editor_content = text_editor::Content::with_text(&content);
    state.markdown_items = markdown::parse(&content).collect();
    state.cursor_position = calculate_cursor_position(&state.editor_content);

    let document = Document::new_markdown_default(&content, &state.dict);
    state.grammar_lints = state.linter.lint(&document);
    state.last_checked_text = content;
}
