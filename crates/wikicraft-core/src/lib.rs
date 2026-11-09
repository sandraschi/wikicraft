//! `wikicraft-core`: Core logic, markdown tree scanning, wiki-link extraction, ADR state machine, and command dispatch.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use craft_biz::bus::Bus;
pub use craft_biz::confine::Confinement;
use craft_biz::mcp::tool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use wikicraft_store::{AdrStatus, Store};

pub const APP: &str = "wikicraft";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Ctx {
    pub store: Store,
    pub confinement: Confinement,
    pub bus: Bus,
    pub actor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Status,
    PageSave {
        id: String,
        title: String,
        content: String,
    },
    PageGet {
        id: String,
    },
    PageList,
    PageDelete {
        id: String,
    },
    PageSearch {
        query: String,
    },
    PageBacklinks {
        id: String,
    },
    AdrSave {
        id: String,
        title: String,
        status: String,
        context: String,
        decision: String,
        consequences: String,
        superseded_by: Option<String>,
    },
    AdrGet {
        id: String,
    },
    AdrList,
    TreeImport {
        path: PathBuf,
    },
    InboxDrain,
}

fn s<'a>(a: &'a Value, k: &str) -> Result<&'a str, String> {
    a.get(k)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string argument {k:?}"))
}

fn opt_s(a: &Value, k: &str) -> Option<String> {
    a.get(k)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|v| !v.trim().is_empty())
}

impl Command {
    pub fn from_tool(name: &str, a: &Value) -> Result<Command, String> {
        let name = craft_biz::mcp::normalize_tool_name(name);
        Ok(match name.as_str() {
            "wiki_status" => Command::Status,
            "wiki_page_save" => Command::PageSave {
                id: s(a, "id")?.to_string(),
                title: s(a, "title")?.to_string(),
                content: s(a, "content")?.to_string(),
            },
            "wiki_page_get" => Command::PageGet {
                id: s(a, "id")?.to_string(),
            },
            "wiki_page_list" => Command::PageList,
            "wiki_page_delete" => Command::PageDelete {
                id: s(a, "id")?.to_string(),
            },
            "wiki_page_search" => Command::PageSearch {
                query: s(a, "query")?.to_string(),
            },
            "wiki_page_backlinks" => Command::PageBacklinks {
                id: s(a, "id")?.to_string(),
            },
            "wiki_adr_save" => Command::AdrSave {
                id: s(a, "id")?.to_string(),
                title: s(a, "title")?.to_string(),
                status: opt_s(a, "status").unwrap_or_else(|| "proposed".to_string()),
                context: opt_s(a, "context").unwrap_or_default(),
                decision: opt_s(a, "decision").unwrap_or_default(),
                consequences: opt_s(a, "consequences").unwrap_or_default(),
                superseded_by: opt_s(a, "superseded_by"),
            },
            "wiki_adr_get" => Command::AdrGet {
                id: s(a, "id")?.to_string(),
            },
            "wiki_adr_list" => Command::AdrList,
            "wiki_tree_import" => {
                let p = s(a, "path")?;
                Command::TreeImport {
                    path: PathBuf::from(p),
                }
            }
            "wiki_inbox_drain" => Command::InboxDrain,
            other => return Err(format!("unknown wikicraft tool: {other}")),
        })
    }
}

/// Extract `[[target]]` or `[[target|label]]` links from markdown text.
pub fn extract_wiki_links(text: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("[[") {
        rest = &rest[start + 2..];
        if let Some(end) = rest.find("]]") {
            let target_raw = &rest[..end];
            let clean = match target_raw.split_once('|') {
                Some((target, _)) => target.trim(),
                None => target_raw.trim(),
            };
            if !clean.is_empty() && !links.contains(&clean.to_string()) {
                links.push(clean.to_string());
            }
            rest = &rest[end + 2..];
        } else {
            break;
        }
    }
    links
}

/// Recursively collect markdown files from a folder.
fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    collect_markdown_files(&path, out);
                }
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown") {
                        out.push(path);
                    }
                }
            }
        }
    }
}

pub fn dispatch(cmd: &Command, ctx: &mut Ctx) -> Result<Value, String> {
    match cmd {
        Command::Status => {
            let stats = ctx.store.stats().map_err(|e| e.to_string())?;
            Ok(json!({
                "app": APP,
                "version": VERSION,
                "pages_count": stats.page_count,
                "adrs_count": stats.adr_count,
                "links_count": stats.link_count,
                "actor": ctx.actor,
                "status": "ok"
            }))
        }
        Command::PageSave { id, title, content } => {
            let page = ctx
                .store
                .save_page(id, title, content)
                .map_err(|e| e.to_string())?;

            let links = extract_wiki_links(content);
            let _ = ctx.store.save_links(id, &links);

            Ok(json!({
                "saved": true,
                "page": page,
                "outgoing_links": links
            }))
        }
        Command::PageGet { id } => {
            let page = ctx.store.get_page(id).map_err(|e| e.to_string())?;
            let backlinks = ctx.store.get_backlinks(id).unwrap_or_default();
            let outgoing = ctx.store.get_outgoing_links(id).unwrap_or_default();
            Ok(json!({
                "page": page,
                "backlinks": backlinks,
                "outgoing_links": outgoing
            }))
        }
        Command::PageList => {
            let list = ctx.store.list_pages().map_err(|e| e.to_string())?;
            Ok(json!({
                "pages": list,
                "count": list.len()
            }))
        }
        Command::PageDelete { id } => {
            let deleted = ctx.store.delete_page(id).map_err(|e| e.to_string())?;
            Ok(json!({
                "id": id,
                "deleted": deleted
            }))
        }
        Command::PageSearch { query } => {
            let results = ctx.store.search_pages(query).map_err(|e| e.to_string())?;
            Ok(json!({
                "query": query,
                "results": results,
                "count": results.len()
            }))
        }
        Command::PageBacklinks { id } => {
            let backlinks = ctx.store.get_backlinks(id).map_err(|e| e.to_string())?;
            Ok(json!({
                "id": id,
                "backlinks": backlinks,
                "count": backlinks.len()
            }))
        }
        Command::AdrSave {
            id,
            title,
            status,
            context,
            decision,
            consequences,
            superseded_by,
        } => {
            let parsed_status = AdrStatus::parse(status).map_err(|e| e.to_string())?;
            let adr = ctx
                .store
                .save_adr(
                    id,
                    title,
                    &parsed_status,
                    context,
                    decision,
                    consequences,
                    superseded_by.as_deref(),
                )
                .map_err(|e| e.to_string())?;

            // Emit bus signal if accepted or superseded
            if parsed_status == AdrStatus::Accepted {
                let _ = ctx.bus.emit(
                    "wikicraft",
                    "*",
                    "adr.accepted",
                    json!({
                        "id": adr.id,
                        "title": adr.title,
                        "decision": adr.decision
                    }),
                );
            }

            Ok(json!({
                "saved": true,
                "adr": adr
            }))
        }
        Command::AdrGet { id } => {
            let adr = ctx.store.get_adr(id).map_err(|e| e.to_string())?;
            Ok(json!({ "adr": adr }))
        }
        Command::AdrList => {
            let list = ctx.store.list_adrs().map_err(|e| e.to_string())?;
            Ok(json!({
                "adrs": list,
                "count": list.len()
            }))
        }
        Command::TreeImport { path } => {
            let path = ctx
                .confinement
                .check(&path.to_string_lossy(), false)
                .map_err(|e| e.to_string())?;
            let mut files = Vec::new();
            collect_markdown_files(&path, &mut files);

            let mut imported = 0usize;
            for f in &files {
                if let Ok(content) = std::fs::read_to_string(f) {
                    let rel = f
                        .strip_prefix(&path)
                        .unwrap_or(f)
                        .to_string_lossy()
                        .replace('\\', "/");
                    let title = f
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&rel)
                        .to_string();

                    if ctx.store.save_page(&rel, &title, &content).is_ok() {
                        let links = extract_wiki_links(&content);
                        let _ = ctx.store.save_links(&rel, &links);
                        imported += 1;
                    }
                }
            }

            Ok(json!({
                "scanned_files": files.len(),
                "imported_pages": imported
            }))
        }
        Command::InboxDrain => {
            let pending = ctx.bus.pending("wikicraft").map_err(|e| e.to_string())?;
            let mut applied = 0usize;
            for evt in &pending {
                ctx.bus
                    .ack("wikicraft", &evt.id)
                    .map_err(|e| e.to_string())?;
                applied += 1;
            }
            Ok(json!({ "applied": applied, "rejected": 0 }))
        }
    }
}

pub fn tools() -> Vec<Value> {
    vec![
        tool(
            "wiki_status",
            "Report wikicraft status, page counts, ADR counts, and health",
            json!({}),
            &[],
        ),
        tool(
            "wiki_page_save",
            "Create or update a wiki documentation page",
            json!({
                "id": { "type": "string" },
                "title": { "type": "string" },
                "content": { "type": "string" }
            }),
            &["id", "title", "content"],
        ),
        tool(
            "wiki_page_get",
            "Retrieve a wiki page by ID with backlinks and outgoing links",
            json!({
                "id": { "type": "string" }
            }),
            &["id"],
        ),
        tool("wiki_page_list", "List all documentation pages", json!({}), &[]),
        tool(
            "wiki_page_delete",
            "Delete a documentation page by ID",
            json!({
                "id": { "type": "string" }
            }),
            &["id"],
        ),
        tool(
            "wiki_page_search",
            "Full-text search (SQLite FTS5) across documentation pages",
            json!({
                "query": { "type": "string" }
            }),
            &["query"],
        ),
        tool(
            "wiki_page_backlinks",
            "Find all pages that link to the given page ID",
            json!({
                "id": { "type": "string" }
            }),
            &["id"],
        ),
        tool(
            "wiki_adr_save",
            "Create or update an Architecture Decision Record (ADR)",
            json!({
                "id": { "type": "string" },
                "title": { "type": "string" },
                "status": { "type": "string", "enum": ["proposed", "accepted", "rejected", "superseded"] },
                "context": { "type": "string" },
                "decision": { "type": "string" },
                "consequences": { "type": "string" },
                "superseded_by": { "type": "string" }
            }),
            &["id", "title", "decision"],
        ),
        tool(
            "wiki_adr_get",
            "Get details of an Architecture Decision Record (ADR)",
            json!({
                "id": { "type": "string" }
            }),
            &["id"],
        ),
        tool("wiki_adr_list", "List all Architecture Decision Records (ADRs)", json!({}), &[]),
        tool(
            "wiki_tree_import",
            "Recursively import a directory of Markdown documentation files into the wiki",
            json!({
                "path": { "type": "string" }
            }),
            &["path"],
        ),
        tool(
            "wiki_inbox_drain",
            "Acknowledge and clear pending inbound signals in the wikicraft inbox",
            json!({}),
            &[],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_wiki_links() {
        let text = "Check [[architecture/overview]] and [[standards/AGENTS|Agent Guide]]. Also see [[architecture/overview]] again.";
        let links = extract_wiki_links(text);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], "architecture/overview");
        assert_eq!(links[1], "standards/AGENTS");
    }

    #[test]
    fn test_dispatch_page_and_adr_flow() {
        let temp_dir = std::env::temp_dir().join(format!("wiki_core_test_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let store = Store::open(&temp_dir.join("test.db")).unwrap();

        let mut ctx = Ctx {
            store,
            confinement: Confinement(None),
            bus: Bus::new(temp_dir.join("bus")),
            actor: "test".to_string(),
        };

        let save_cmd = Command::PageSave {
            id: "guide/getting-started".to_string(),
            title: "Getting Started".to_string(),
            content: "Welcome to [[guide/installation]].".to_string(),
        };
        let res = dispatch(&save_cmd, &mut ctx).unwrap();
        assert!(res.get("saved").unwrap().as_bool().unwrap());

        let search_cmd = Command::PageSearch {
            query: "welcome".to_string(),
        };
        let search_res = dispatch(&search_cmd, &mut ctx).unwrap();
        assert_eq!(search_res.get("count").unwrap().as_u64().unwrap(), 1);

        let adr_cmd = Command::AdrSave {
            id: "ADR-001".to_string(),
            title: "Use Rust".to_string(),
            status: "accepted".to_string(),
            context: "Performance and safety".to_string(),
            decision: "Adopt Rust 2021 edition".to_string(),
            consequences: "Memory safety without GC".to_string(),
            superseded_by: None,
        };
        let adr_res = dispatch(&adr_cmd, &mut ctx).unwrap();
        assert!(adr_res.get("saved").unwrap().as_bool().unwrap());
    }
}
