use crate::app::primary_shortcut_modifier_pressed;
use crate::app::types::{Message, State, ViewMode};
use crate::app::ui::styles::{
    TEXT_LOW, command_input_style, content_card_style, danger_button_style, floating_panel_style,
    footer_style, ghost_button_style, raw_editor_style, segmented_button_style, top_header_style,
};
use harper_core::linting::{Lint, LintKind};
use iced::keyboard;
use iced::widget::text::Highlighter;
use iced::widget::{
    button, column, container, markdown, row, scrollable, svg, text, text_editor, text_input,
};
use iced::{Element, Fill};
use std::ops::Range;

const APPLY_ICON_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/apply.svg");
const PANEL_HEIGHT_COLLAPSED: u32 = 40;
const PANEL_HEIGHT_EXPANDED: u32 = 200;

pub(crate) fn build_editor_area(state: &State) -> Element<'_, Message> {
    let title_input = text_input("Conversation title", &state.current_title)
        .on_input(Message::TitleChanged)
        .on_submit(Message::SaveTitle)
        .padding([12, 16])
        .style(command_input_style)
        .width(Fill);

    let title_actions: Element<'_, Message> = if state.selected_in_trash {
        text("This conversation is in Trash")
            .size(12)
            .color(TEXT_LOW)
            .into()
    } else if let Some(id) = state.selected_conversation {
        button(text("Move to Trash").size(12))
            .on_press(Message::MoveConversationToTrash(id))
            .padding([8, 10])
            .style(danger_button_style)
            .into()
    } else {
        text("").into()
    };

    let title_bar = row![title_input, title_actions]
        .spacing(8)
        .padding([14, 18])
        .align_y(iced::Center);

    let apply_icon = svg(APPLY_ICON_PATH)
        .width(16)
        .height(16)
        .style(|_theme, _status| svg::Style {
            color: Some(iced::Color::WHITE),
        });

    let apply_all_button = button(
        row![apply_icon, text("Apply All").size(13)]
            .spacing(6)
            .align_y(iced::Center),
    )
    .on_press_maybe((!state.grammar_lints.is_empty()).then_some(Message::ApplyAllSuggestions))
    .padding([8, 12])
    .style(segmented_button_style(true));

    let mode_buttons = row![
        button(text("Raw Code").size(13))
            .on_press(Message::SetViewMode(ViewMode::RawCode))
            .padding([6, 12])
            .style(segmented_button_style(state.view_mode == ViewMode::RawCode)),
        button(text("Both Views").size(13))
            .on_press(Message::SetViewMode(ViewMode::BothViews))
            .padding([6, 12])
            .style(segmented_button_style(
                state.view_mode == ViewMode::BothViews
            )),
        button(text("Rendered").size(13))
            .on_press(Message::SetViewMode(ViewMode::RenderedView))
            .padding([6, 12])
            .style(segmented_button_style(
                state.view_mode == ViewMode::RenderedView
            )),
        apply_all_button,
    ]
    .spacing(10)
    .padding([12, 18]);

    let editor_content: Element<'_, Message> = match state.view_mode {
        ViewMode::RawCode => build_raw_editor(state),
        ViewMode::BothViews => {
            let rendered = scrollable(
                markdown::view(&state.markdown_items, iced::Theme::TokyoNight)
                    .map(Message::MarkdownLinkClicked),
            )
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(8)
                    .scroller_width(8)
                    .margin(2)
                    .spacing(10),
            ))
            .height(Fill);

            row![
                build_raw_editor(state),
                container(rendered)
                    .width(Fill)
                    .height(Fill)
                    .padding([18, 18])
                    .style(content_card_style)
            ]
            .spacing(14)
            .into()
        }
        ViewMode::RenderedView => {
            let rendered = scrollable(
                markdown::view(&state.markdown_items, iced::Theme::TokyoNight)
                    .map(Message::MarkdownLinkClicked),
            )
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::new()
                    .width(8)
                    .scroller_width(8)
                    .margin(2)
                    .spacing(10),
            ))
            .height(Fill);

            container(rendered)
                .width(Fill)
                .height(Fill)
                .padding([18, 18])
                .style(content_card_style)
                .into()
        }
    };

    let collapse_progress = state
        .errors_panel_animation
        .interpolate(0.0_f32, 1.0_f32, state.now)
        .clamp(0.0, 1.0);
    let panel_span = (PANEL_HEIGHT_EXPANDED - PANEL_HEIGHT_COLLAPSED) as f32;
    let panel_height =
        (PANEL_HEIGHT_EXPANDED as f32 - panel_span * collapse_progress).round() as u32;
    let use_compact_layout = panel_height <= PANEL_HEIGHT_COLLAPSED + 18;

    let toggle_icon = if use_compact_layout { "▲" } else { "▼" };
    let issue_count = state.grammar_lints.len();
    let status_text = if issue_count == 0 {
        "No issues found ✓".to_string()
    } else {
        format!("{} issue(s)", issue_count)
    };

    let panel_header = row![
        text(status_text).size(13).color(if issue_count == 0 {
            [0.58, 0.86, 0.62]
        } else {
            [1.0, 0.84, 0.58]
        }),
        button(text(toggle_icon).size(12))
            .on_press(Message::ToggleErrorsPanel)
            .padding([4, 8])
            .style(ghost_button_style),
    ]
    .spacing(12)
    .align_y(iced::Center);

    let suggestions_panel = if use_compact_layout {
        container(panel_header).padding(12).width(Fill)
    } else if state.grammar_lints.is_empty() {
        container(
            column![
                panel_header,
                text("Your grammar is perfect!").size(12).color(TEXT_LOW)
            ]
            .spacing(8),
        )
        .padding([12, 14])
        .width(Fill)
    } else {
        let lint_list = column(
            state
                .grammar_lints
                .iter()
                .enumerate()
                .take(10)
                .map(|(index, lint)| {
                    let message = lint.message.clone();
                    let suggestion_text = if let Some(suggestion) = lint.suggestions.first() {
                        format!("→ {}", suggestion)
                    } else {
                        String::from("(no suggestion)")
                    };
                    let issue_color = lint_color_for_kind(lint.lint_kind);
                    let apply_button = button(text("Apply").size(11))
                        .on_press_maybe(
                            lint.suggestions
                                .first()
                                .map(|_| Message::ApplyLintSuggestion(index)),
                        )
                        .padding([6, 10])
                        .style(segmented_button_style(true));
                    let dismiss_button = button(text("Dismiss").size(11))
                        .on_press(Message::DismissLint(index))
                        .padding([6, 10])
                        .style(ghost_button_style);

                    Element::from(
                        column![
                            row![
                                text(message).size(13).color(issue_color),
                                apply_button,
                                dismiss_button
                            ]
                            .align_y(iced::Center)
                            .spacing(8),
                            text(suggestion_text).size(12).color(TEXT_LOW),
                        ]
                        .spacing(4)
                        .padding([10, 12]),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .spacing(8);

        container(column![panel_header, scrollable(lint_list).height(Fill),].spacing(8))
            .padding([12, 14])
            .width(Fill)
    };

    let shortcut_modifier = if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    };
    let shortcuts_hint = text(format!("Press {shortcut_modifier} + K to show shortcuts"))
        .size(12)
        .color(TEXT_LOW);
    let cursor_text = text(format!(
        "Ln {}, Col {}",
        state.cursor_position.0, state.cursor_position.1
    ))
    .size(12)
    .color(TEXT_LOW);

    let footer = container(
        row![
            shortcuts_hint,
            container(cursor_text)
                .width(Fill)
                .align_x(iced::alignment::Horizontal::Right)
        ]
        .align_y(iced::Center)
        .spacing(12),
    )
    .padding([8, 12])
    .style(footer_style);

    let full_editor_area = column![
        container(title_bar).width(Fill).style(top_header_style),
        container(mode_buttons).width(Fill).style(top_header_style),
        container(editor_content).height(Fill),
        container(suggestions_panel)
            .width(Fill)
            .height(panel_height)
            .style(floating_panel_style),
        footer,
    ]
    .spacing(10);

    container(full_editor_area).width(Fill).height(Fill).into()
}

fn build_raw_editor(state: &State) -> Element<'_, Message> {
    let highlight_settings =
        build_lint_highlight_settings(&state.last_checked_text, &state.grammar_lints);
    let editor = text_editor(&state.editor_content)
        .on_action(Message::EditorAction)
        .key_binding(|key_press| {
            if is_app_shortcut_keypress(&key_press) {
                Some(text_editor::Binding::Sequence(Vec::new()))
            } else {
                text_editor::Binding::from_key_press(key_press)
            }
        })
        .highlight_with::<LintHighlighter>(highlight_settings, lint_highlight_format)
        .style(raw_editor_style)
        .height(Fill)
        .padding([14, 14]);

    container(editor)
        .width(Fill)
        .height(Fill)
        .padding([12, 12])
        .style(content_card_style)
        .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LintHighlight {
    Error,
    Warning,
    Suggestion,
}

#[derive(Debug, Clone, PartialEq)]
struct LintHighlightSettings {
    lines: Vec<Vec<(Range<usize>, LintHighlight)>>,
}

struct LintHighlighter {
    settings: LintHighlightSettings,
    current_line: usize,
}

impl Highlighter for LintHighlighter {
    type Settings = LintHighlightSettings;
    type Highlight = LintHighlight;
    type Iterator<'a> = std::vec::IntoIter<(Range<usize>, Self::Highlight)>;

    fn new(settings: &Self::Settings) -> Self {
        Self {
            settings: settings.clone(),
            current_line: 0,
        }
    }

    fn update(&mut self, new_settings: &Self::Settings) {
        self.settings = new_settings.clone();
        // Reset cursor whenever settings change so a fresh render starts at line 0.
        self.current_line = 0;
    }

    fn change_line(&mut self, line: usize) {
        self.current_line = line;
    }

    fn highlight_line(&mut self, _line: &str) -> Self::Iterator<'_> {
        let items = self
            .settings
            .lines
            .get(self.current_line)
            .cloned()
            .unwrap_or_default();
        self.current_line += 1;
        items.into_iter()
    }

    fn current_line(&self) -> usize {
        self.current_line
    }
}

fn build_lint_highlight_settings(text: &str, lints: &[Lint]) -> LintHighlightSettings {
    #[derive(Clone, Copy)]
    struct LineRange {
        byte_start: usize,
        byte_end: usize,
    }

    fn offset_to_byte(offset: usize, text: &str, char_to_byte: &[usize]) -> Option<usize> {
        // Harper spans are character offsets; map those first.
        if let Some(byte_offset) = char_to_byte.get(offset).copied() {
            return Some(byte_offset);
        }

        // Fallback for any byte-based span source.
        if offset <= text.len() && text.is_char_boundary(offset) {
            return Some(offset);
        }

        None
    }

    let mut char_to_byte = Vec::with_capacity(text.chars().count() + 1);
    for (byte_index, _) in text.char_indices() {
        char_to_byte.push(byte_index);
    }
    char_to_byte.push(text.len());

    let mut lines_meta = Vec::new();
    let mut line_start = 0usize;
    for (byte_index, ch) in text.char_indices() {
        if ch == '\n' {
            lines_meta.push(LineRange {
                byte_start: line_start,
                byte_end: byte_index,
            });
            line_start = byte_index + ch.len_utf8();
        }
    }
    lines_meta.push(LineRange {
        byte_start: line_start,
        byte_end: text.len(),
    });

    let mut lines = vec![Vec::new(); lines_meta.len().max(1)];

    for lint in lints {
        if lint.span.is_empty() {
            continue;
        }
        let Some(start_byte) = offset_to_byte(lint.span.start, text, &char_to_byte) else {
            continue;
        };
        let Some(end_byte) = offset_to_byte(lint.span.end, text, &char_to_byte) else {
            continue;
        };
        if end_byte <= start_byte {
            continue;
        }

        let level = lint_level_from_kind(lint.lint_kind);
        let start_line = lines_meta.partition_point(|line| line.byte_end <= start_byte);
        let end_line = lines_meta.partition_point(|line| line.byte_start < end_byte);

        for (line_index, line) in lines_meta[start_line..end_line].iter().enumerate() {
            let local_start = start_byte.saturating_sub(line.byte_start);
            let local_end = end_byte.min(line.byte_end).saturating_sub(line.byte_start);
            if local_start < local_end {
                lines[start_line + line_index].push((local_start..local_end, level));
            }
        }
    }

    for line in &mut lines {
        line.sort_by_key(|(span, _)| span.start);
    }

    LintHighlightSettings { lines }
}

fn is_app_shortcut_keypress(key_press: &text_editor::KeyPress) -> bool {
    if !primary_shortcut_modifier_pressed(key_press.modifiers) {
        return false;
    }

    if let Some(character) = key_press
        .key
        .to_latin(key_press.physical_key)
        .map(|c| c.to_ascii_lowercase())
    {
        return matches!(character, 'f' | 'b' | 'n' | 'k' | '1' | '2' | '3');
    }

    matches!(
        key_press.physical_key,
        keyboard::key::Physical::Code(keyboard::key::Code::Digit1)
            | keyboard::key::Physical::Code(keyboard::key::Code::Digit2)
            | keyboard::key::Physical::Code(keyboard::key::Code::Digit3)
            | keyboard::key::Physical::Code(keyboard::key::Code::KeyK)
    )
}

fn lint_highlight_format(
    highlight: &LintHighlight,
    _theme: &iced::Theme,
) -> iced_core::text::highlighter::Format<iced::Font> {
    let mut format = iced_core::text::highlighter::Format::default();
    format.color = Some(match highlight {
        LintHighlight::Error => iced::Color::from_rgb(0.98, 0.35, 0.35),
        LintHighlight::Warning => iced::Color::from_rgb(1.0, 0.78, 0.35),
        LintHighlight::Suggestion => iced::Color::from_rgb(0.45, 0.82, 1.0),
    });
    format
}

fn lint_level_from_kind(kind: LintKind) -> LintHighlight {
    match kind {
        LintKind::Enhancement | LintKind::Style | LintKind::Readability => {
            LintHighlight::Suggestion
        }
        LintKind::Usage
        | LintKind::WordChoice
        | LintKind::Regionalism
        | LintKind::Repetition
        | LintKind::Redundancy
        | LintKind::Formatting
        | LintKind::Miscellaneous
        | LintKind::Eggcorn => LintHighlight::Warning,
        _ => LintHighlight::Error,
    }
}

fn lint_color_for_kind(kind: LintKind) -> [f32; 3] {
    match lint_level_from_kind(kind) {
        LintHighlight::Error => [1.0, 0.7, 0.7],
        LintHighlight::Warning => [1.0, 0.82, 0.55],
        LintHighlight::Suggestion => [0.65, 0.88, 1.0],
    }
}

#[cfg(test)]
mod tests {
    use super::{LintHighlight, build_lint_highlight_settings};
    use harper_core::Span;
    use harper_core::linting::{Lint, LintKind};

    fn lint(span: std::ops::Range<usize>, kind: LintKind) -> Lint {
        Lint {
            span: Span::from(span),
            lint_kind: kind,
            ..Lint::default()
        }
    }

    #[test]
    fn highlight_maps_single_line_span() {
        let settings =
            build_lint_highlight_settings("alpha beta", &[lint(6..10, LintKind::Grammar)]);
        assert_eq!(settings.lines.len(), 1);
        assert_eq!(settings.lines[0], vec![(6..10, LintHighlight::Error)]);
    }

    #[test]
    fn highlight_splits_multiline_span_by_line() {
        let settings =
            build_lint_highlight_settings("abc\ndefg\nh", &[lint(2..7, LintKind::Grammar)]);
        assert_eq!(settings.lines.len(), 3);
        assert_eq!(settings.lines[0], vec![(2..3, LintHighlight::Error)]);
        assert_eq!(settings.lines[1], vec![(0..3, LintHighlight::Error)]);
        assert!(settings.lines[2].is_empty());
    }

    #[test]
    fn highlight_uses_char_offsets_when_unicode_exists_before_span() {
        let text = "🙂🙂 like seperate";
        let target_start_char = text.find("seperate").unwrap();
        let target_start_char = text[..target_start_char].chars().count();
        let target_end_char = target_start_char + "seperate".chars().count();

        let settings = build_lint_highlight_settings(
            text,
            &[lint(target_start_char..target_end_char, LintKind::Grammar)],
        );
        assert_eq!(settings.lines.len(), 1);

        let (range, _) = &settings.lines[0][0];
        let expected_start = text.find("seperate").unwrap();
        let expected_end = expected_start + "seperate".len();
        assert_eq!(range.start, expected_start);
        assert_eq!(range.end, expected_end);
    }
}
