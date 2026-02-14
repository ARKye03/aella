use std::path::PathBuf;
use turso::{Builder, Connection, Result};

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
                std::fs::create_dir_all(parent).expect("Failed to create database directory");
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
        let _ = conn
            .execute(
                "ALTER TABLE conversations ADD COLUMN trashed_at DATETIME",
                (),
            )
            .await;

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
}

#[derive(Debug, Clone)]
pub struct ConversationData {
    pub id: i64,
    pub title: String,
    pub content: String,
}
