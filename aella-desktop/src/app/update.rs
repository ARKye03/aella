use crate::app::types::{Conversation, Message, State, ViewMode};
use crate::app::{
    AUTOSAVE_DEBOUNCE, GRAMMAR_CHECK_DEBOUNCE, primary_shortcut_modifier_pressed,
    sidebar_search_input_id,
};
use crate::db::Database;
use harper_core::Dialect;
use harper_core::Document;
use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use iced::Task;
use iced::keyboard;
use iced::keyboard::Key;
use iced::keyboard::key::Physical;
use iced::widget::{markdown, operation, text_editor};
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::sync::Arc;

thread_local! {
    static GRAMMAR_LINTER_CACHE: RefCell<Option<LintGroup>> = const { RefCell::new(None) };
}

pub(crate) fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::KeyboardEvent(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            repeat,
            ..
        }) => {
            if repeat {
                return Task::none();
            }

            state.now = std::time::Instant::now();

            if state.shortcuts_help_open && matches!(key, Key::Named(keyboard::key::Named::Escape))
            {
                set_shortcuts_help_open(state, false);
                return Task::none();
            }

            if !primary_shortcut_modifier_pressed(modifiers) {
                return Task::none();
            }

            if is_shortcuts_help_shortcut(key.to_latin(physical_key), physical_key) {
                remove_shortcut_text_input_artifact(state, 'k');
                set_shortcuts_help_open(state, !state.shortcuts_help_open);
                return Task::none();
            }

            if let Some(character) = key.to_latin(physical_key).map(|c| c.to_ascii_lowercase()) {
                match character {
                    'f' => {
                        remove_shortcut_text_input_artifact(state, character);
                        state.sidebar_collapsed = false;
                        return operation::focus(sidebar_search_input_id());
                    }
                    '1' => {
                        remove_shortcut_text_input_artifact(state, character);
                        state.view_mode = ViewMode::RawCode;
                    }
                    '2' => {
                        remove_shortcut_text_input_artifact(state, character);
                        state.view_mode = ViewMode::BothViews;
                    }
                    '3' => {
                        remove_shortcut_text_input_artifact(state, character);
                        state.view_mode = ViewMode::RenderedView;
                    }
                    'b' => {
                        remove_shortcut_text_input_artifact(state, character);
                        if modifiers.shift() {
                            state.errors_panel_collapsed = !state.errors_panel_collapsed;
                        } else {
                            state.sidebar_collapsed = !state.sidebar_collapsed;
                        }
                    }
                    'n' => {
                        remove_shortcut_text_input_artifact(state, character);
                        return update(state, Message::NewConversation);
                    }
                    _ => {}
                }
            } else if key_matches_digit(physical_key) {
                match physical_key {
                    Physical::Code(keyboard::key::Code::Digit1) => {
                        remove_shortcut_text_input_artifact(state, '1');
                        state.view_mode = ViewMode::RawCode;
                    }
                    Physical::Code(keyboard::key::Code::Digit2) => {
                        remove_shortcut_text_input_artifact(state, '2');
                        state.view_mode = ViewMode::BothViews;
                    }
                    Physical::Code(keyboard::key::Code::Digit3) => {
                        remove_shortcut_text_input_artifact(state, '3');
                        state.view_mode = ViewMode::RenderedView
                    }
                    _ => {}
                }
            }
        }
        Message::KeyboardEvent(_) => {}
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
                        conv.title_search = normalize_for_search(&state.current_title);
                    }
                } else if let Some(conv) = state
                    .conversations
                    .iter_mut()
                    .find(|conv| conv.id == selected_id)
                {
                    conv.title = state.current_title.clone();
                    conv.title_search = normalize_for_search(&state.current_title);
                }
            }
            return schedule_autosave_task(state);
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
                        if let Err(err) = db_clone.create_conversation(&title, &content).await {
                            eprintln!("[aella] failed to create conversation: {err}");
                        }
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
                        title_search: normalize_for_search(&format!("New Conversation {}", new_id)),
                        preview_search: normalize_for_search("Start writing..."),
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
                        if let Err(err) = db_clone.trash_conversation(id).await {
                            eprintln!("[aella] failed to move conversation {id} to trash: {err}");
                        }
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
                        if let Err(err) = db_clone.restore_conversation(id).await {
                            eprintln!("[aella] failed to restore conversation {id}: {err}");
                        }
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
                        if let Err(err) = db_clone.delete_conversation(id).await {
                            eprintln!("[aella] failed to delete conversation {id}: {err}");
                        }
                    },
                    |_| Message::RefreshLists,
                );
            }
        }
        Message::ApplyLintSuggestion(index) => {
            apply_single_suggestion(state, index);
            return schedule_autosave_task(state);
        }
        Message::ApplyAllSuggestions => {
            apply_all_suggestions(state);
            return schedule_autosave_task(state);
        }
        Message::EditorAction(action) => {
            let text_before = state.editor_content.text();
            state.editor_content.perform(action);

            let text = state.editor_content.text();
            let text_changed = text != text_before;

            state.cursor_position = calculate_cursor_position(&state.editor_content);
            if text_changed {
                if text != state.last_checked_text {
                    state.grammar_lints.clear();
                    state.markdown_dirty = true;
                }
                return Task::batch(vec![
                    schedule_autosave_task(state),
                    schedule_grammar_check_task(state, text),
                ]);
            }
        }
        Message::ToggleSidebar => {
            state.sidebar_collapsed = !state.sidebar_collapsed;
        }
        Message::SetViewMode(mode) => {
            state.view_mode = mode;
            parse_markdown_if_needed(state);
        }
        Message::MarkdownLinkClicked(_url) => {}
        Message::ToggleErrorsPanel => {
            state.errors_panel_collapsed = !state.errors_panel_collapsed;
        }
        Message::CloseShortcutsHelp => {
            state.now = std::time::Instant::now();
            set_shortcuts_help_open(state, false);
        }
        Message::Tick(now) => {
            state.now = now;
        }
        Message::DatabaseInitialized(db) => {
            state.database = Some(db);
            return refresh_lists_task(state);
        }
        Message::ConversationsLoaded(db_conversations) => {
            let had_selection = state.selected_conversation;
            state.conversations = db_conversations.iter().map(conversation_from_db).collect();

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
            state.trashed_conversations =
                db_conversations.iter().map(conversation_from_db).collect();

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
            state.grammar_check_generation = state.grammar_check_generation.saturating_add(1);
            state.current_title = conv.title;
            state.editor_content = text_editor::Content::with_text(&conv.content);
            state.markdown_dirty = true;
            state.last_checked_text = conv.content.clone();
            parse_markdown_if_needed(state);
            return run_grammar_check_task(state.dict.clone(), conv.content);
        }
        Message::RefreshLists => {
            return refresh_lists_task(state);
        }
        Message::AutosaveDue(generation) => {
            if generation == state.autosave_generation {
                return save_current_conversation_task(state);
            }
        }
        Message::GrammarCheckDebounced {
            generation,
            content,
        } => {
            if should_run_grammar_check(state, generation, &content) {
                return run_grammar_check_task(state.dict.clone(), content);
            }
        }
        Message::GrammarChecked { content, lints } => {
            if state.editor_content.text() == content {
                state.grammar_lints = lints;
                state.last_checked_text = content;
                state.markdown_dirty = true;
                parse_markdown_if_needed(state);
            }
        }
        Message::Noop => {}
        Message::ConversationSaved => {}
    }
    Task::none()
}

pub(crate) fn initialize_database_task() -> Task<Message> {
    Task::perform(
        async {
            match Database::new().await {
                Ok(db) => Some(db),
                Err(err) => {
                    eprintln!("[aella] failed to initialize database: {err}");
                    None
                }
            }
        },
        |db_opt| {
            if let Some(db) = db_opt {
                Message::DatabaseInitialized(db)
            } else {
                Message::Noop
            }
        },
    )
}

fn save_current_conversation_task(state: &State) -> Task<Message> {
    if let (Some(database), Some(selected_id)) = (&state.database, state.selected_conversation) {
        let db_clone = database.clone();
        let title = state.current_title.clone();
        let content = state.editor_content.text();
        return Task::perform(
            async move {
                if let Err(err) = db_clone
                    .update_conversation(selected_id, &title, &content)
                    .await
                {
                    eprintln!("[aella] failed to save conversation {selected_id}: {err}");
                }
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
            async move {
                match db_clone.get_conversation(id).await {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("[aella] failed to load conversation {id}: {err}");
                        None
                    }
                }
            },
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
                async move {
                    match active_db.get_all_conversations().await {
                        Ok(conversations) => conversations,
                        Err(err) => {
                            eprintln!("[aella] failed to load conversations: {err}");
                            Vec::new()
                        }
                    }
                },
                Message::ConversationsLoaded,
            ),
            Task::perform(
                async move {
                    match trash_db.get_trashed_conversations().await {
                        Ok(conversations) => conversations,
                        Err(err) => {
                            eprintln!("[aella] failed to load trashed conversations: {err}");
                            Vec::new()
                        }
                    }
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
            let document = Document::new_markdown_default(&content, &dict_for_doc);
            let lints = GRAMMAR_LINTER_CACHE.with(|slot| {
                let mut slot = slot.borrow_mut();
                let linter =
                    slot.get_or_insert_with(|| LintGroup::new_curated(dict, Dialect::American));
                linter.lint(&document)
            });
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

    let mut result = String::new();
    let mut count = 0usize;
    let mut truncated = false;

    for ch in content.chars() {
        if count == 50 {
            truncated = true;
            break;
        }
        result.push(ch);
        count += 1;
    }

    if truncated {
        result.push_str("...");
    }

    result
}

fn clear_loaded_conversation(state: &mut State) {
    state.grammar_check_generation = state.grammar_check_generation.saturating_add(1);
    state.selected_conversation = None;
    state.current_title = String::from("Untitled");
    state.editor_content = text_editor::Content::new();
    state.markdown_items = Vec::new();
    state.markdown_dirty = false;
    state.grammar_lints = Vec::new();
    state.last_checked_text.clear();
}

fn key_matches_digit(physical_key: Physical) -> bool {
    matches!(
        physical_key,
        Physical::Code(keyboard::key::Code::Digit1)
            | Physical::Code(keyboard::key::Code::Digit2)
            | Physical::Code(keyboard::key::Code::Digit3)
    )
}

fn is_shortcuts_help_shortcut(key_char: Option<char>, physical_key: Physical) -> bool {
    matches!(key_char.map(|c| c.to_ascii_lowercase()), Some('k'))
        || matches!(physical_key, Physical::Code(keyboard::key::Code::KeyK))
}

fn set_shortcuts_help_open(state: &mut State, open: bool) {
    if state.shortcuts_help_open == open {
        return;
    }

    state.shortcuts_help_open = open;
    state.shortcuts_help_animation.go_mut(open, state.now);
}

fn remove_shortcut_text_input_artifact(state: &mut State, shortcut_char: char) {
    let changed_search = strip_trailing_shortcut_char(&mut state.search_query, shortcut_char);
    if changed_search {
        return;
    }

    let changed_title = strip_trailing_shortcut_char(&mut state.current_title, shortcut_char);

    if !changed_title {
        return;
    }

    if let Some(selected_id) = state.selected_conversation {
        if state.selected_in_trash {
            if let Some(conv) = state
                .trashed_conversations
                .iter_mut()
                .find(|conv| conv.id == selected_id)
            {
                conv.title = state.current_title.clone();
                conv.title_search = normalize_for_search(&state.current_title);
            }
        } else if let Some(conv) = state
            .conversations
            .iter_mut()
            .find(|conv| conv.id == selected_id)
        {
            conv.title = state.current_title.clone();
            conv.title_search = normalize_for_search(&state.current_title);
        }
    }
}

fn schedule_autosave_task(state: &mut State) -> Task<Message> {
    state.autosave_generation = state.autosave_generation.saturating_add(1);
    let generation = state.autosave_generation;
    Task::perform(
        async move {
            tokio::time::sleep(AUTOSAVE_DEBOUNCE).await;
            generation
        },
        Message::AutosaveDue,
    )
}

fn schedule_grammar_check_task(state: &mut State, content: String) -> Task<Message> {
    state.grammar_check_generation = state.grammar_check_generation.saturating_add(1);
    let generation = state.grammar_check_generation;
    Task::perform(
        async move {
            tokio::time::sleep(GRAMMAR_CHECK_DEBOUNCE).await;
            (generation, content)
        },
        |(generation, content)| Message::GrammarCheckDebounced {
            generation,
            content,
        },
    )
}

fn parse_markdown_if_needed(state: &mut State) {
    if !state.markdown_dirty {
        return;
    }
    if matches!(state.view_mode, ViewMode::RawCode) {
        return;
    }

    state.markdown_items = markdown::parse(&state.last_checked_text).collect();
    state.markdown_dirty = false;
}

fn should_run_grammar_check(state: &State, generation: u64, content: &str) -> bool {
    generation == state.grammar_check_generation
        && content != state.last_checked_text
        && state.editor_content.text() == content
}

fn conversation_from_db(conv: &crate::db::ConversationData) -> Conversation {
    let preview = preview(&conv.content);
    Conversation {
        id: conv.id,
        title: conv.title.clone(),
        title_search: normalize_for_search(&conv.title),
        preview_search: normalize_for_search(&preview),
        preview,
    }
}

fn normalize_for_search(text: &str) -> String {
    text.trim().to_lowercase()
}

// Whoever thought about tests within core files, deserves to burn in hell (Subjective opinion btw)
#[cfg(test)]
mod tests {
    use super::should_run_grammar_check;
    use crate::app::types::State;
    use iced::widget::text_editor;

    #[test]
    fn debounce_gate_accepts_current_payload() {
        let mut state = State::default();
        state.grammar_check_generation = 42;
        state.last_checked_text = String::from("old");
        state.editor_content = text_editor::Content::with_text("new");

        assert!(should_run_grammar_check(&state, 42, "new"));
    }

    #[test]
    fn debounce_gate_rejects_stale_generation() {
        let mut state = State::default();
        state.grammar_check_generation = 42;
        state.last_checked_text = String::from("old");
        state.editor_content = text_editor::Content::with_text("new");

        assert!(!should_run_grammar_check(&state, 41, "new"));
    }

    #[test]
    fn debounce_gate_rejects_already_checked_content() {
        let mut state = State::default();
        state.grammar_check_generation = 42;
        state.last_checked_text = String::from("new");
        state.editor_content = text_editor::Content::with_text("new");

        assert!(!should_run_grammar_check(&state, 42, "new"));
    }

    #[test]
    fn debounce_gate_rejects_if_editor_moved_on() {
        let mut state = State::default();
        state.grammar_check_generation = 42;
        state.last_checked_text = String::from("old");
        state.editor_content = text_editor::Content::with_text("newest");

        assert!(!should_run_grammar_check(&state, 42, "new"));
    }
}

fn strip_trailing_shortcut_char(value: &mut String, shortcut_char: char) -> bool {
    let mut chars = value.chars();
    let Some(last) = chars.next_back() else {
        return false;
    };

    let should_strip = if shortcut_char.is_ascii_alphabetic() {
        last.eq_ignore_ascii_case(&shortcut_char)
    } else {
        last == shortcut_char
    };

    if should_strip {
        value.pop();
    }

    should_strip
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
