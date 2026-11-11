//! `wikicraft-store`: SQLite WAL-backed documentation, ADR lifecycle, and FTS5 search store.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WikiError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Page not found: {0}")]
    PageNotFound(String),
    #[error("ADR not found: {0}")]
    AdrNotFound(String),
    #[error("Invalid ADR status: {0}")]
    InvalidStatus(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSummary {
    pub id: String,
    pub title: String,
    pub updated_at: String,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
}

impl AdrStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
        }
    }

    pub fn parse(s: &str) -> Result<Self, WikiError> {
        match s.to_lowercase().trim() {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "superseded" => Ok(Self::Superseded),
            other => Err(WikiError::InvalidStatus(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adr {
    pub id: String,
    pub title: String,
    pub status: String,
    pub context: String,
    pub decision: String,
    pub consequences: String,
    pub superseded_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiStats {
    pub page_count: usize,
    pub adr_count: usize,
    pub link_count: usize,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, WikiError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;

             CREATE TABLE IF NOT EXISTS pages (
                 id TEXT PRIMARY KEY,
                 title TEXT NOT NULL,
                 content TEXT NOT NULL,
                 created_at TEXT NOT NULL,
                 updated_at TEXT NOT NULL,
                 version INTEGER NOT NULL DEFAULT 1
             );

             CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(
                 id UNINDEXED,
                 title,
                 content
             );

             CREATE TABLE IF NOT EXISTS adrs (
                 id TEXT PRIMARY KEY,
                 title TEXT NOT NULL,
                 status TEXT NOT NULL,
                 context TEXT NOT NULL,
                 decision TEXT NOT NULL,
                 consequences TEXT NOT NULL,
                 superseded_by TEXT,
                 created_at TEXT NOT NULL,
                 updated_at TEXT NOT NULL
             );

             CREATE TABLE IF NOT EXISTS links (
                 source_id TEXT NOT NULL,
                 target_id TEXT NOT NULL,
                 PRIMARY KEY (source_id, target_id)
             );",
        )?;

        Ok(Self { conn })
    }

    pub fn save_page(&mut self, id: &str, title: &str, content: &str) -> Result<Page, WikiError> {
        let now = Utc::now().to_rfc3339();
        let existing: Option<(String, i64)> = self
            .conn
            .query_row(
                "SELECT created_at, version FROM pages WHERE id = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;

        let (created_at, version) = match existing {
            Some((c, v)) => (c, v + 1),
            None => (now.clone(), 1),
        };

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT OR REPLACE INTO pages (id, title, content, created_at, updated_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, title, content, created_at, now, version],
        )?;

        tx.execute("DELETE FROM pages_fts WHERE id = ?1", params![id])?;
        tx.execute(
            "INSERT INTO pages_fts (id, title, content) VALUES (?1, ?2, ?3)",
            params![id, title, content],
        )?;

        tx.commit()?;

        Ok(Page {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            created_at,
            updated_at: now,
            version,
        })
    }

    pub fn get_page(&self, id: &str) -> Result<Page, WikiError> {
        self.conn
            .query_row(
                "SELECT id, title, content, created_at, updated_at, version FROM pages WHERE id = ?1",
                params![id],
                |r| {
                    Ok(Page {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        content: r.get(2)?,
                        created_at: r.get(3)?,
                        updated_at: r.get(4)?,
                        version: r.get(5)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| WikiError::PageNotFound(id.to_string()))
    }

    pub fn list_pages(&self) -> Result<Vec<PageSummary>, WikiError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, updated_at, version FROM pages ORDER BY title ASC")?;
        let rows = stmt.query_map([], |r| {
            Ok(PageSummary {
                id: r.get(0)?,
                title: r.get(1)?,
                updated_at: r.get(2)?,
                version: r.get(3)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_page(&mut self, id: &str) -> Result<bool, WikiError> {
        let tx = self.conn.transaction()?;
        let affected = tx.execute("DELETE FROM pages WHERE id = ?1", params![id])?;
        tx.execute("DELETE FROM pages_fts WHERE id = ?1", params![id])?;
        tx.execute(
            "DELETE FROM links WHERE source_id = ?1 OR target_id = ?1",
            params![id],
        )?;
        tx.commit()?;
        Ok(affected > 0)
    }

    pub fn search_pages(&self, query: &str) -> Result<Vec<SearchResult>, WikiError> {
        let sanitized = query.replace('"', "\"\"").replace('*', "");
        if sanitized.trim().is_empty() {
            return Ok(Vec::new());
        }
        let match_expr = format!("\"{sanitized}\"*");

        let mut stmt = self.conn.prepare(
            "SELECT id, title, snippet(pages_fts, 2, '<b>', '</b>', '...', 12)
             FROM pages_fts
             WHERE pages_fts MATCH ?1
             ORDER BY rank
             LIMIT 25",
        )?;

        let rows = stmt.query_map(params![match_expr], |r| {
            Ok(SearchResult {
                id: r.get(0)?,
                title: r.get(1)?,
                snippet: r.get(2)?,
            })
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    pub fn save_adr(
        &mut self,
        id: &str,
        title: &str,
        status: &AdrStatus,
        context: &str,
        decision: &str,
        consequences: &str,
        superseded_by: Option<&str>,
    ) -> Result<Adr, WikiError> {
        let now = Utc::now().to_rfc3339();
        let existing_created: Option<String> = self
            .conn
            .query_row(
                "SELECT created_at FROM adrs WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .optional()?;

        let created_at = existing_created.unwrap_or_else(|| now.clone());
        let status_str = status.as_str();

        self.conn.execute(
            "INSERT OR REPLACE INTO adrs (id, title, status, context, decision, consequences, superseded_by, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![id, title, status_str, context, decision, consequences, superseded_by, created_at, now],
        )?;

        Ok(Adr {
            id: id.to_string(),
            title: title.to_string(),
            status: status_str.to_string(),
            context: context.to_string(),
            decision: decision.to_string(),
            consequences: consequences.to_string(),
            superseded_by: superseded_by.map(str::to_string),
            created_at,
            updated_at: now,
        })
    }

    pub fn get_adr(&self, id: &str) -> Result<Adr, WikiError> {
        self.conn
            .query_row(
                "SELECT id, title, status, context, decision, consequences, superseded_by, created_at, updated_at
                 FROM adrs WHERE id = ?1",
                params![id],
                |r| {
                    Ok(Adr {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        status: r.get(2)?,
                        context: r.get(3)?,
                        decision: r.get(4)?,
                        consequences: r.get(5)?,
                        superseded_by: r.get(6)?,
                        created_at: r.get(7)?,
                        updated_at: r.get(8)?,
                    })
                },
            )
            .optional()?
            .ok_or_else(|| WikiError::AdrNotFound(id.to_string()))
    }

    pub fn list_adrs(&self) -> Result<Vec<Adr>, WikiError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, status, context, decision, consequences, superseded_by, created_at, updated_at
             FROM adrs ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Adr {
                id: r.get(0)?,
                title: r.get(1)?,
                status: r.get(2)?,
                context: r.get(3)?,
                decision: r.get(4)?,
                consequences: r.get(5)?,
                superseded_by: r.get(6)?,
                created_at: r.get(7)?,
                updated_at: r.get(8)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn save_links(&mut self, source_id: &str, target_ids: &[String]) -> Result<(), WikiError> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM links WHERE source_id = ?1", params![source_id])?;
        for t in target_ids {
            tx.execute(
                "INSERT OR IGNORE INTO links (source_id, target_id) VALUES (?1, ?2)",
                params![source_id, t],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_backlinks(&self, target_id: &str) -> Result<Vec<String>, WikiError> {
        let mut stmt = self
            .conn
            .prepare("SELECT source_id FROM links WHERE target_id = ?1 ORDER BY source_id ASC")?;
        let rows = stmt.query_map(params![target_id], |r| r.get(0))?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_outgoing_links(&self, source_id: &str) -> Result<Vec<String>, WikiError> {
        let mut stmt = self
            .conn
            .prepare("SELECT target_id FROM links WHERE source_id = ?1 ORDER BY target_id ASC")?;
        let rows = stmt.query_map(params![source_id], |r| r.get(0))?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn stats(&self) -> Result<WikiStats, WikiError> {
        let page_count: i64 = self
            .conn
            .query_row("SELECT count(*) FROM pages", [], |r| r.get(0))?;
        let adr_count: i64 = self
            .conn
            .query_row("SELECT count(*) FROM adrs", [], |r| r.get(0))?;
        let link_count: i64 = self
            .conn
            .query_row("SELECT count(*) FROM links", [], |r| r.get(0))?;
        Ok(WikiStats {
            page_count: page_count as usize,
            adr_count: adr_count as usize,
            link_count: link_count as usize,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_page_crud_and_fts_search() {
        let temp_dir = std::env::temp_dir().join(format!("wiki_test_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_file = temp_dir.join("test.db");

        let mut store = Store::open(&db_file).unwrap();
        let page = store
            .save_page(
                "architecture/overview",
                "Fleet Architecture Overview",
                "This document covers the fleet architecture and fast SQLite replication.",
            )
            .unwrap();

        assert_eq!(page.id, "architecture/overview");
        assert_eq!(page.version, 1);

        let fetched = store.get_page("architecture/overview").unwrap();
        assert_eq!(fetched.title, "Fleet Architecture Overview");

        let search = store.search_pages("replication").unwrap();
        assert_eq!(search.len(), 1);
        assert_eq!(search[0].id, "architecture/overview");

        let stats = store.stats().unwrap();
        assert_eq!(stats.page_count, 1);
    }

    #[test]
    fn store_adr_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("wiki_adr_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_file = temp_dir.join("test_adr.db");

        let mut store = Store::open(&db_file).unwrap();
        let adr = store
            .save_adr(
                "ADR-001",
                "Adopt SQLite WAL for Backoffice",
                &AdrStatus::Proposed,
                "Need local-first low-latency storage.",
                "Adopt SQLite WAL mode.",
                "Zero network latency, ACID compliance.",
                None,
            )
            .unwrap();

        assert_eq!(adr.status, "proposed");

        let updated = store
            .save_adr(
                "ADR-001",
                "Adopt SQLite WAL for Backoffice",
                &AdrStatus::Accepted,
                "Need local-first low-latency storage.",
                "Adopt SQLite WAL mode.",
                "Zero network latency, ACID compliance.",
                None,
            )
            .unwrap();

        assert_eq!(updated.status, "accepted");
    }
}
