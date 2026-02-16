use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationData {
    pub id: i64,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliRun {
    pub id: i64,
    pub input_source: String,
    pub input_length: i64,
    pub output_length: i64,
    pub corrections_count: i64,
    pub passes_count: i64,
    pub execution_time_ms: i64,
    pub created_at: String,
    pub original_text: Option<String>,
    pub corrected_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CliRunMetrics<'a> {
    pub input_source: &'a str,
    pub input_length: usize,
    pub output_length: usize,
    pub corrections_count: usize,
    pub passes_count: usize,
    pub execution_time_ms: u64,
}
