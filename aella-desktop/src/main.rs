use iced::widget::{button, column, container, row, scrollable, svg, text, text_editor, text_input};
use iced::{Center, Color, Element, Fill, Task};

fn main() -> iced::Result {
    iced::run(update, view)
}

struct State {
    search_query: String,
    conversations: Vec<Conversation>,
    selected_conversation: Option<usize>,
    editor_content: text_editor::Content,
    sidebar_collapsed: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            conversations: vec![
                Conversation {
                    id: 1,
                    title: String::from("Welcome to Aella"),
                    preview: String::from("Getting started with grammar checking..."),
                },
                Conversation {
                    id: 2,
                    title: String::from("Project Ideas"),
                    preview: String::from("Brainstorming new features..."),
                },
                Conversation {
                    id: 3,
                    title: String::from("Meeting Notes"),
                    preview: String::from("Discussion points from today..."),
                },
            ],
            selected_conversation: Some(0),
            editor_content: text_editor::Content::with_text(
                "# Welcome to Aella\n\n\
                Start typing to check your grammar in real-time.\n\n\
                This is a markdown editor with built-in grammar checking powered by Harper.",
            ),
            sidebar_collapsed: false,
        }
    }
}

#[derive(Debug, Clone)]
struct Conversation {
    id: usize,
    title: String,
    preview: String,
}

#[derive(Debug, Clone)]
enum Message {
    SearchChanged(String),
    ConversationSelected(usize),
    NewConversation,
    EditorAction(text_editor::Action),
    ToggleSidebar,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::SearchChanged(query) => {
            state.search_query = query;
        }
        Message::ConversationSelected(index) => {
            state.selected_conversation = Some(index);
        }
        Message::NewConversation => {
            let new_id = state.conversations.len() + 1;
            state.conversations.insert(
                0,
                Conversation {
                    id: new_id,
                    title: format!("New Conversation {}", new_id),
                    preview: String::from("Start writing..."),
                },
            );
            state.selected_conversation = Some(0);
            state.editor_content = text_editor::Content::new();
        }
        Message::EditorAction(action) => {
            state.editor_content.perform(action);
        }
        Message::ToggleSidebar => {
            state.sidebar_collapsed = !state.sidebar_collapsed;
        }
    }
    Task::none()
}

fn view(state: &State) -> Element<'_, Message> {
    let sidebar = build_sidebar(state);
    let editor = build_editor(state);

    let content = row![sidebar, editor].spacing(0);

    container(content).width(Fill).height(Fill).into()
}

fn build_sidebar(state: &State) -> Element<'_, Message> {
    let sidebar_width = if state.sidebar_collapsed { 60 } else { 300 };

    // Toggle button
    let toggle_icon = if state.sidebar_collapsed { "☰" } else { "←" };
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

    let conversation_list = scrollable(column(
        filtered_conversations
            .iter()
            .enumerate()
            .map(|(index, conv)| {
                let conv_button = button(
                    column![
                        text(&conv.title),
                        text(&conv.preview).size(12).color([0.6, 0.6, 0.6]),
                    ]
                    .spacing(4)
                    .padding([12, 16]),
                )
                .on_press(Message::ConversationSelected(index))
                .width(Fill);

                Element::from(conv_button)
            })
            .collect::<Vec<_>>(),
    )
    .spacing(4)
    .padding([8, 12]));

    let sidebar_content = column![toggle_button, search, new_button, conversation_list].spacing(8);

    container(sidebar_content)
        .width(sidebar_width)
        .height(Fill)
        .padding(8)
        .into()
}

fn build_editor(state: &State) -> Element<'_, Message> {
    let editor = text_editor(&state.editor_content)
        .on_action(Message::EditorAction)
        .height(Fill)
        .padding(24);

    container(editor).width(Fill).height(Fill).into()
}
