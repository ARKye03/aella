use clap::{Parser, Subcommand};
use colored::Colorize;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "aella")]
#[command(about = "Grammar checking CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check and correct grammar in text
    Check {
        /// Input file (reads from stdin if not provided)
        file: Option<String>,

        /// Output file (prints to stdout if not provided)
        #[arg(short, long)]
        output: Option<String>,

        /// Show diff instead of corrected text
        #[arg(long)]
        diff: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Save original and corrected text to database
        #[arg(long)]
        save_content: bool,

        /// Batch process multiple files
        #[arg(long)]
        batch: bool,
    },

    /// View CLI usage history
    History {
        /// Number of runs to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            file,
            output,
            diff,
            json,
            save_content,
            batch,
        } => {
            if batch {
                handle_batch_check(file, output, diff, json, save_content).await;
            } else {
                handle_check(file, output, diff, json, save_content).await;
            }
        }
        Commands::History { limit } => {
            handle_history(limit).await;
        }
    }
}

async fn handle_check(
    file: Option<String>,
    output: Option<String>,
    diff: bool,
    json: bool,
    save_content: bool,
) {
    let start = std::time::Instant::now();

    // Read input
    let (input_text, input_source) = if let Some(path) = &file {
        match fs::read_to_string(path) {
            Ok(content) => (content, format!("file:{}", path)),
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        let mut buffer = String::new();
        match io::stdin().read_to_string(&mut buffer) {
            Ok(_) => (buffer, "stdin".to_string()),
            Err(e) => {
                eprintln!("Error reading stdin: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Apply corrections
    let result = aella_core::apply_all_corrections(&input_text, harper_core::Dialect::American);

    let execution_time = start.elapsed().as_millis() as u64;

    // Save to database
    if let Ok(db) = aella_core::Database::new().await {
        let original = if save_content {
            Some(result.original_text.as_str())
        } else {
            None
        };
        let corrected = if save_content {
            Some(result.corrected_text.as_str())
        } else {
            None
        };

        let metrics = aella_core::CliRunMetrics {
            input_source: &input_source,
            input_length: input_text.len(),
            output_length: result.corrected_text.len(),
            corrections_count: result.corrections_count,
            passes_count: result.passes_count,
            execution_time_ms: execution_time,
        };
        let _ = db.create_cli_run(&metrics, original, corrected).await;
    }

    // Output results
    if json {
        output_json(&result, execution_time);
    } else if diff {
        output_diff(&result.original_text, &result.corrected_text);
    } else {
        output_text(&result.corrected_text, output);
    }
}

fn output_text(text: &str, output_file: Option<String>) {
    if let Some(path) = output_file {
        if let Err(e) = fs::write(&path, text) {
            eprintln!("Error writing output file: {}", e);
            std::process::exit(1);
        }
    } else {
        println!("{}", text);
    }
}

fn output_json(result: &aella_core::CorrectionResult, execution_time: u64) {
    let json = serde_json::json!({
        "original": result.original_text,
        "corrected": result.corrected_text,
        "corrections_count": result.corrections_count,
        "passes_count": result.passes_count,
        "execution_time_ms": execution_time,
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}

fn output_diff(original: &str, corrected: &str) {
    let diff = TextDiff::from_lines(original, corrected);

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-".red(),
            ChangeTag::Insert => "+".green(),
            ChangeTag::Equal => " ".normal(),
        };
        print!("{}{}", sign, change);
    }
}

async fn handle_batch_check(
    pattern: Option<String>,
    output_dir: Option<String>,
    diff: bool,
    json: bool,
    save_content: bool,
) {
    let pattern = match pattern {
        Some(p) => p,
        None => {
            eprintln!("Error: Pattern required for batch mode");
            std::process::exit(1);
        }
    };

    let paths: Vec<_> = match glob::glob(&pattern) {
        Ok(paths) => paths.filter_map(Result::ok).collect(),
        Err(e) => {
            eprintln!("Invalid glob pattern: {}", e);
            std::process::exit(1);
        }
    };

    if paths.is_empty() {
        eprintln!("No files matched pattern: {}", pattern);
        std::process::exit(1);
    }

    println!("Processing {} files...", paths.len());

    // Create output directory if specified
    if let Some(dir) = &output_dir {
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!("Error creating output directory: {}", e);
            std::process::exit(1);
        }
    }

    for path in paths {
        let path_str = path.to_string_lossy().to_string();
        println!("Processing: {}", path_str);

        let output = if let Some(dir) = &output_dir {
            let filename = path.file_name().unwrap().to_string_lossy();
            Some(format!("{}/{}", dir, filename))
        } else {
            None
        };

        handle_check(Some(path_str), output, diff, json, save_content).await;
    }
}

async fn handle_history(limit: usize) {
    let db = match aella_core::Database::new().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return;
        }
    };

    let runs = match db.get_cli_runs(limit).await {
        Ok(runs) => runs,
        Err(e) => {
            eprintln!("Failed to fetch CLI runs: {}", e);
            return;
        }
    };

    if runs.is_empty() {
        println!("No CLI runs found.");
        return;
    }

    println!(
        "{:<5} {:<20} {:<20} {:<12} {:<10} {:<10}",
        "ID", "Date", "Source", "Corrections", "Passes", "Time (ms)"
    );
    println!("{}", "-".repeat(85));

    for run in runs {
        println!(
            "{:<5} {:<20} {:<20} {:<12} {:<10} {:<10}",
            run.id,
            truncate(&run.created_at, 20),
            truncate(&run.input_source, 20),
            run.corrections_count,
            run.passes_count,
            run.execution_time_ms
        );
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
