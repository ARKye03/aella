use std::path::PathBuf;
use turso::{Builder, Connection, Result};

use crate::types::{CliRun, ConversationData};

#[derive(Clone, Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path();

        #[cfg(debug_assertions)]
        eprintln!("[aella] database path: {}", db_path.display());

        if !cfg!(debug_assertions) {
            if let Some(parent) = db_path.parent() {
                if let Err(err) = std::fs::create_dir_all(parent) {
                    eprintln!(
                        "[aella] failed to create database directory {}: {err}",
                        parent.display()
                    );
                }
            }
        }

        let db_path_str = db_path.to_string_lossy();
        let db = Builder::new_local(&db_path_str).build().await?;
        let conn = db.connect()?;

        // Create conversations table
        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                trashed_at DATETIME,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"#,
            (),
        )
        .await?;

        // Lightweight migration for older local DBs.
        if let Err(err) = conn
            .execute(
                "ALTER TABLE conversations ADD COLUMN trashed_at DATETIME",
                (),
            )
            .await
        {
            // Existing databases already have this column.
            if !err.to_string().contains("duplicate column name") {
                eprintln!("[aella] migration failed when adding trashed_at column: {err}");
            }
        }

        // Create CLI runs table
        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS cli_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                input_source TEXT NOT NULL,
                input_length INTEGER NOT NULL,
                output_length INTEGER NOT NULL,
                corrections_count INTEGER NOT NULL,
                passes_count INTEGER NOT NULL,
                execution_time_ms INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                original_text TEXT,
                corrected_text TEXT
            )"#,
            (),
        )
        .await?;

        Ok(Self { conn })
    }

    fn get_db_path() -> PathBuf {
        if cfg!(debug_assertions) {
            return PathBuf::from("aella.db");
        }

        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("aella");
        path.push("aella.db");
        path
    }

    pub async fn create_conversation(&self, title: &str, content: &str) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO conversations (title, content) VALUES (?1, ?2)",
                (title, content),
            )
            .await?;

        // Get the last inserted id
        let mut rows = self.conn.query("SELECT last_insert_rowid()", ()).await?;

        if let Some(row) = rows.next().await? {
            let id = row.get_value(0)?;
            Ok(*id.as_integer().unwrap_or(&0))
        } else {
            Ok(0)
        }
    }

    pub async fn get_conversation(&self, id: i64) -> Result<Option<ConversationData>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, title, content FROM conversations WHERE id = ?1",
                (id,),
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(ConversationData {
                id: row.get_value(0)?.as_integer().copied().unwrap_or(0),
                title: row
                    .get_value(1)?
                    .as_text()
                    .unwrap_or(&String::new())
                    .clone(),
                content: row
                    .get_value(2)?
                    .as_text()
                    .unwrap_or(&String::new())
                    .clone(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_conversations(&self) -> Result<Vec<ConversationData>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, title, content FROM conversations WHERE trashed_at IS NULL ORDER BY updated_at DESC",
                (),
            )
            .await?;

        let mut conversations = Vec::new();

        while let Some(row) = rows.next().await? {
            conversations.push(ConversationData {
                id: row.get_value(0)?.as_integer().copied().unwrap_or(0),
                title: row
                    .get_value(1)?
                    .as_text()
                    .unwrap_or(&String::new())
                    .clone(),
                content: row
                    .get_value(2)?
                    .as_text()
                    .unwrap_or(&String::new())
                    .clone(),
            });
        }

        Ok(conversations)
    }

    pub async fn get_trashed_conversations(&self) -> Result<Vec<ConversationData>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, title, content FROM conversations WHERE trashed_at IS NOT NULL ORDER BY trashed_at DESC",
                (),
            )
            .await?;

        let mut conversations = Vec::new();

        while let Some(row) = rows.next().await? {
            conversations.push(ConversationData {
                id: row.get_value(0)?.as_integer().copied().unwrap_or(0),
                title: row
                    .get_value(1)?
                    .as_text()
                    .unwrap_or(&String::new())
                    .clone(),
                content: row
                    .get_value(2)?
                    .as_text()
                    .unwrap_or(&String::new())
                    .clone(),
            });
        }

        Ok(conversations)
    }

    pub async fn update_conversation(&self, id: i64, title: &str, content: &str) -> Result<()> {
        self.conn
            .execute(
                "UPDATE conversations SET title = ?1, content = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
                (title, content, id),
            )
            .await?;

        Ok(())
    }

    pub async fn delete_conversation(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM conversations WHERE id = ?1", (id,))
            .await?;

        Ok(())
    }

    pub async fn trash_conversation(&self, id: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE conversations SET trashed_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
                (id,),
            )
            .await?;

        Ok(())
    }

    pub async fn restore_conversation(&self, id: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE conversations SET trashed_at = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
                (id,),
            )
            .await?;

        Ok(())
    }

    pub async fn create_cli_run(
        &self,
        input_source: &str,
        input_length: usize,
        output_length: usize,
        corrections_count: usize,
        passes_count: usize,
        execution_time_ms: u64,
        original_text: Option<&str>,
        corrected_text: Option<&str>,
    ) -> Result<i64> {
        self.conn
            .execute(
                r#"INSERT INTO cli_runs
                   (input_source, input_length, output_length, corrections_count,
                    passes_count, execution_time_ms, original_text, corrected_text)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
                (
                    input_source,
                    input_length as i64,
                    output_length as i64,
                    corrections_count as i64,
                    passes_count as i64,
                    execution_time_ms as i64,
                    original_text,
                    corrected_text,
                ),
            )
            .await?;

        // Get the last inserted id
        let mut rows = self.conn.query("SELECT last_insert_rowid()", ()).await?;

        if let Some(row) = rows.next().await? {
            let id = row.get_value(0)?;
            Ok(*id.as_integer().unwrap_or(&0))
        } else {
            Ok(0)
        }
    }

    pub async fn get_cli_runs(&self, limit: usize) -> Result<Vec<CliRun>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, input_source, input_length, output_length, corrections_count,
                        passes_count, execution_time_ms, created_at, original_text, corrected_text
                 FROM cli_runs
                 ORDER BY created_at DESC
                 LIMIT ?1",
                (limit as i64,),
            )
            .await?;

        let mut runs = Vec::new();

        while let Some(row) = rows.next().await? {
            runs.push(CliRun {
                id: row.get_value(0)?.as_integer().copied().unwrap_or(0),
                input_source: row
                    .get_value(1)?
                    .as_text()
                    .unwrap_or(&String::new())
                    .clone(),
                input_length: row.get_value(2)?.as_integer().copied().unwrap_or(0),
                output_length: row.get_value(3)?.as_integer().copied().unwrap_or(0),
                corrections_count: row.get_value(4)?.as_integer().copied().unwrap_or(0),
                passes_count: row.get_value(5)?.as_integer().copied().unwrap_or(0),
                execution_time_ms: row.get_value(6)?.as_integer().copied().unwrap_or(0),
                created_at: row
                    .get_value(7)?
                    .as_text()
                    .unwrap_or(&String::new())
                    .clone(),
                original_text: row.get_value(8)?.as_text().map(|s| s.clone()),
                corrected_text: row.get_value(9)?.as_text().map(|s| s.clone()),
            });
        }

        Ok(runs)
    }
}
