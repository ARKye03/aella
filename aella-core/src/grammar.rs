use harper_core::{
    linting::{LintGroup, Linter},
    spell::FstDictionary,
    Dialect, Document,
};
use std::cmp::Reverse;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct CorrectionResult {
    pub original_text: String,
    pub corrected_text: String,
    pub corrections_count: usize,
    pub passes_count: usize,
}

pub fn apply_all_corrections(text: &str, dialect: Dialect) -> CorrectionResult {
    const MAX_PASSES: usize = 8;

    let dict = FstDictionary::curated();
    let mut linter = LintGroup::new_curated(dict.clone(), dialect);
    let mut content = text.to_string();
    let mut total_corrections = 0;
    let mut passes = 0;

    for pass in 0..MAX_PASSES {
        let document = Document::new_markdown_default(&content, &dict);
        let lints = linter.lint(&document);

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

        total_corrections += edits.len();
        passes = pass + 1;

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

    CorrectionResult {
        original_text: text.to_string(),
        corrected_text: content,
        corrections_count: total_corrections,
        passes_count: passes,
    }
}
