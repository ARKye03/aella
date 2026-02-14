use turso::{Builder, Connection, Result};

#[derive(Clone, Debug)]
pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db = Builder::new_local("aella.db").build().await?;
        let conn = db.connect()?;

        // Create conversations table
        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )"#,
            (),
        )
        .await?;

        Ok(Self { conn })
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
                "SELECT id, title, content FROM conversations ORDER BY updated_at DESC",
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
}

#[derive(Debug, Clone)]
pub struct ConversationData {
    pub id: i64,
    pub title: String,
    pub content: String,
}
