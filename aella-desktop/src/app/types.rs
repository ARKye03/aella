use aella_core::{ConversationData, Database};
use harper_core::Dialect;
use harper_core::Document;
use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use iced::keyboard;
use iced::widget::{markdown, text_editor};
use iced_core::animation::{Animation, Easing};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ViewMode {
    RawCode,
    BothViews,
    RenderedView,
}

#[derive(Debug, Clone)]
pub(crate) struct Conversation {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) preview: String,
    pub(crate) title_search: String,
    pub(crate) preview_search: String,
}

pub(crate) struct State {
    pub(crate) search_query: String,
    pub(crate) conversations: Vec<Conversation>,
    pub(crate) trashed_conversations: Vec<Conversation>,
    pub(crate) selected_conversation: Option<i64>,
    pub(crate) selected_in_trash: bool,
    pub(crate) current_title: String,
    pub(crate) editor_content: text_editor::Content,
    pub(crate) sidebar_collapsed: bool,
    pub(crate) sidebar_animation: Animation<bool>,
    pub(crate) linter: LintGroup,
    pub(crate) grammar_lints: Vec<harper_core::linting::Lint>,
    pub(crate) view_mode: ViewMode,
    pub(crate) markdown_items: Vec<markdown::Item>,
    pub(crate) cursor_position: (usize, usize), // (line, column)
    pub(crate) errors_panel_collapsed: bool,
    pub(crate) shortcuts_help_open: bool,
    pub(crate) shortcuts_help_animation: Animation<bool>,
    pub(crate) now: Instant,
    pub(crate) database: Option<Database>,
    pub(crate) dict: Arc<FstDictionary>, // Reuse dictionary instead of creating on every keystroke
    pub(crate) last_checked_text: String, // Track last text to avoid redundant grammar checks
    pub(crate) autosave_generation: u64,
    pub(crate) grammar_check_generation: u64,
    pub(crate) markdown_dirty: bool,
    pub(crate) dismissed_lint_keys: HashSet<u64>,
}

impl Default for State {
    fn default() -> Self {
        let dict = FstDictionary::curated();
        let mut linter = LintGroup::new_curated(dict.clone(), Dialect::American);
        let initial_text = "# Welcome to Aella\n\n\
                Start typing to check your grammar in real-time.\n\n\
                This is a markdown editor with built-in grammar checking powered by Harper.";

        let initial_document = Document::new_markdown_default(initial_text, &dict);
        let grammar_lints = linter.lint(&initial_document);
        let markdown_items: Vec<markdown::Item> = markdown::parse(initial_text).collect();

        Self {
            now: Instant::now(),
            search_query: String::new(),
            conversations: Vec::new(),
            trashed_conversations: Vec::new(),
            selected_conversation: None,
            selected_in_trash: false,
            current_title: String::from("Untitled"),
            editor_content: text_editor::Content::with_text(initial_text),
            sidebar_collapsed: false,
            sidebar_animation: Animation::new(false)
                .duration(std::time::Duration::from_millis(220))
                .easing(Easing::EaseOutCubic),
            linter,
            grammar_lints,
            view_mode: ViewMode::BothViews,
            markdown_items,
            cursor_position: (1, 1),
            errors_panel_collapsed: false,
            shortcuts_help_open: false,
            shortcuts_help_animation: Animation::new(false)
                .duration(std::time::Duration::from_millis(180))
                .easing(Easing::EaseOutCubic),
            database: None,
            dict,
            last_checked_text: initial_text.to_string(),
            autosave_generation: 0,
            grammar_check_generation: 0,
            markdown_dirty: false,
            dismissed_lint_keys: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    KeyboardEvent(keyboard::Event),
    SearchChanged(String),
    TitleChanged(String),
    SaveTitle,
    ConversationSelected(i64),
    NewConversation,
    ShowConversations,
    ShowTrash,
    MoveConversationToTrash(i64),
    RestoreConversation(i64),
    DeleteConversationPermanently(i64),
    ApplyLintSuggestion(usize),
    DismissLint(usize),
    ApplyAllSuggestions,
    EditorAction(text_editor::Action),
    ToggleSidebar,
    SetViewMode(ViewMode),
    MarkdownLinkClicked(markdown::Uri),
    ToggleErrorsPanel,
    CloseShortcutsHelp,
    Tick(Instant),
    DatabaseInitialized(Database),
    ConversationsLoaded(Vec<ConversationData>),
    TrashedConversationsLoaded(Vec<ConversationData>),
    ConversationDataLoaded(ConversationData),
    RefreshLists,
    AutosaveDue(u64),
    GrammarCheckDebounced {
        generation: u64,
        content: String,
    },
    GrammarChecked {
        content: String,
        lints: Vec<harper_core::linting::Lint>,
    },
    ConversationSaved,
    Noop,
}
