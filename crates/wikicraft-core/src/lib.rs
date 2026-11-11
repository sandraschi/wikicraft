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
    SeedDemo,
    GhWikiPull {
        repo: String,
    },
    GhWikiPush {
        repo: String,
        message: Option<String>,
    },
    GhWikiStatus {
        repo: String,
    },
    SiteExport {
        out_dir: PathBuf,
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
            "wiki_seed_demo" => Command::SeedDemo,
            "wiki_gh_pull" => Command::GhWikiPull {
                repo: s(a, "repo")?.to_string(),
            },
            "wiki_gh_push" => Command::GhWikiPush {
                repo: s(a, "repo")?.to_string(),
                message: opt_s(a, "message"),
            },
            "wiki_gh_status" => Command::GhWikiStatus {
                repo: s(a, "repo")?.to_string(),
            },
            "wiki_site_export" => {
                let p = s(a, "out_dir")?;
                Command::SiteExport {
                    out_dir: PathBuf::from(p),
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

/// Normalize repo string to `owner/repo` (defaults to `sandraschi/<repo>` if no slash provided).
pub fn normalize_gh_repo(repo: &str) -> String {
    let trimmed = repo.trim();
    if trimmed.contains('/') {
        trimmed.to_string()
    } else {
        format!("sandraschi/{trimmed}")
    }
}

/// Local directory where the GitHub Wiki git repository is cached.
pub fn gh_wiki_local_dir(repo: &str) -> PathBuf {
    let full = normalize_gh_repo(repo);
    let clean = full.replace(['/', '\\'], "_");
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("wikicraft").join("wikis").join(clean)
}

/// Remote git clone URL for a GitHub Wiki repository.
pub fn gh_wiki_remote_url(repo: &str) -> String {
    let full = normalize_gh_repo(repo);
    format!("https://github.com/{full}.wiki.git")
}

/// Pull/clone a GitHub Wiki repository and ingest its markdown pages into SQLite FTS5.
pub fn gh_wiki_pull(repo: &str, ctx: &mut Ctx) -> Result<Value, String> {
    let full_repo = normalize_gh_repo(repo);
    let dir = gh_wiki_local_dir(&full_repo);
    let remote_url = gh_wiki_remote_url(&full_repo);

    if let Some(parent) = dir.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let git_dir = dir.join(".git");
    if git_dir.exists() {
        let out = std::process::Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "pull", "--rebase"])
            .output()
            .map_err(|e| format!("failed to run git pull: {e}"))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!("git pull failed: {err}"));
        }
    } else {
        let out = std::process::Command::new("git")
            .args(["clone", &remote_url, &dir.to_string_lossy()])
            .output()
            .map_err(|e| format!("failed to run git clone: {e}"))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!(
                "git clone failed for {remote_url}: {err}. (Note: GitHub creates the wiki git backend when the first page ('Home') is created via the GitHub web UI)."
            ));
        }
    }

    let mut files = Vec::new();
    collect_markdown_files(&dir, &mut files);

    let mut imported = 0usize;
    for f in &files {
        if let Ok(content) = std::fs::read_to_string(f) {
            let file_stem = f.file_stem().and_then(|s| s.to_str()).unwrap_or("page");
            let title = content
                .lines()
                .find(|l| l.trim().starts_with("# "))
                .map(|l| l.trim().trim_start_matches("# ").trim().to_string())
                .unwrap_or_else(|| file_stem.to_string());

            if ctx.store.save_page(file_stem, &title, &content).is_ok() {
                let links = extract_wiki_links(&content);
                let _ = ctx.store.save_links(file_stem, &links);
                imported += 1;
            }
        }
    }

    Ok(json!({
        "repo": full_repo,
        "pulled": true,
        "scanned_files": files.len(),
        "imported_pages": imported,
        "wiki_dir": dir.to_string_lossy()
    }))
}

/// Export stored pages and ADRs to a GitHub Wiki git repository, generating `_Sidebar.md` and pushing to GitHub.
pub fn gh_wiki_push(repo: &str, custom_msg: Option<&str>, ctx: &mut Ctx) -> Result<Value, String> {
    let full_repo = normalize_gh_repo(repo);
    let dir = gh_wiki_local_dir(&full_repo);
    let remote_url = gh_wiki_remote_url(&full_repo);

    let _ = std::fs::create_dir_all(&dir);

    let git_dir = dir.join(".git");
    if !git_dir.exists() {
        let init_out = std::process::Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "init", "-b", "master"])
            .output()
            .map_err(|e| format!("failed to git init: {e}"))?;
        if !init_out.status.success() {
            let err = String::from_utf8_lossy(&init_out.stderr);
            return Err(format!("git init failed: {err}"));
        }

        let _ = std::process::Command::new("git")
            .args([
                "-C",
                &dir.to_string_lossy(),
                "remote",
                "add",
                "origin",
                &remote_url,
            ])
            .output();
    }

    let pages = ctx.store.list_pages().map_err(|e| e.to_string())?;
    let adrs = ctx.store.list_adrs().map_err(|e| e.to_string())?;

    let mut exported_count = 0usize;
    let mut sidebar_links = Vec::new();

    for p_summary in &pages {
        if let Ok(full_page) = ctx.store.get_page(&p_summary.id) {
            let safe_filename = full_page.id.replace(['/', '\\'], "-");
            let file_path = dir.join(format!("{safe_filename}.md"));
            if std::fs::write(&file_path, &full_page.content).is_ok() {
                exported_count += 1;
                sidebar_links.push(format!("- [[{}|{}]]", safe_filename, full_page.title));
            }
        }
    }

    // Auto-generate _Sidebar.md for GitHub Wiki navigation
    let mut sidebar_content = String::from("### Documentation\n");
    for link in &sidebar_links {
        sidebar_content.push_str(link);
        sidebar_content.push('\n');
    }
    if !adrs.is_empty() {
        sidebar_content.push_str("\n### Architecture Decisions\n");
        for a in &adrs {
            sidebar_content.push_str(&format!("- [[{}|{} ({})]]\n", a.id, a.title, a.status));
            let adr_path = dir.join(format!("{}.md", a.id));
            let adr_body = format!(
                "# {}\n\n**Status**: `{}`\n**Updated**: {}\n\n## Context\n{}\n\n## Decision\n{}\n\n## Consequences\n{}\n",
                a.title, a.status, a.updated_at, a.context, a.decision, a.consequences
            );
            let _ = std::fs::write(&adr_path, adr_body);
        }
    }
    let _ = std::fs::write(dir.join("_Sidebar.md"), sidebar_content);

    // Auto-generate _Footer.md
    let footer_content = "---\n*Maintained with [WikiCraft](https://github.com/sandraschi/wikicraft) — Local-first documentation studio*\n";
    let _ = std::fs::write(dir.join("_Footer.md"), footer_content);

    // Git stage & commit
    let _ = std::process::Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "add", "-A"])
        .output();

    let msg = custom_msg.unwrap_or("docs: sync wiki pages from WikiCraft");
    let _ = std::process::Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "commit", "-m", msg])
        .output();

    // Push to remote (try master, fallback main)
    let push_out = std::process::Command::new("git")
        .args([
            "-C",
            &dir.to_string_lossy(),
            "push",
            "-u",
            "origin",
            "master",
        ])
        .output()
        .map_err(|e| format!("failed to run git push: {e}"))?;

    let (pushed, push_info) = if push_out.status.success() {
        (true, "pushed to origin/master".to_string())
    } else {
        let push_main = std::process::Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "push", "-u", "origin", "main"])
            .output();
        if let Ok(ref pm) = push_main {
            if pm.status.success() {
                (true, "pushed to origin/main".to_string())
            } else {
                let stderr = String::from_utf8_lossy(&push_out.stderr);
                (false, format!("push failed: {stderr}"))
            }
        } else {
            let stderr = String::from_utf8_lossy(&push_out.stderr);
            (false, format!("push failed: {stderr}"))
        }
    };

    Ok(json!({
        "repo": full_repo,
        "pushed": pushed,
        "push_info": push_info,
        "exported_pages": exported_count,
        "adrs_exported": adrs.len(),
        "wiki_dir": dir.to_string_lossy()
    }))
}

/// Inspect local and remote status of a GitHub Wiki.
pub fn gh_wiki_status(repo: &str) -> Result<Value, String> {
    let full_repo = normalize_gh_repo(repo);
    let dir = gh_wiki_local_dir(&full_repo);
    let remote_url = gh_wiki_remote_url(&full_repo);
    let exists = dir.join(".git").exists();

    let mut commit_info = "not initialized".to_string();
    if exists {
        if let Ok(out) = std::process::Command::new("git")
            .args(["-C", &dir.to_string_lossy(), "log", "-1", "--oneline"])
            .output()
        {
            if out.status.success() {
                commit_info = String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }
    }

    Ok(json!({
        "repo": full_repo,
        "remote_url": remote_url,
        "local_dir": dir.to_string_lossy(),
        "initialized": exists,
        "last_commit": commit_info
    }))
}

/// Escape HTML special characters.
pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Convert page ID to safe HTML filename.
pub fn page_id_to_filename(id: &str) -> String {
    let clean = id.replace(['/', '\\', ' '], "__");
    format!("{clean}.html")
}

fn process_basic_inlines(text: &str) -> String {
    let escaped = escape_html(text);
    let mut res = String::new();
    let mut parts = escaped.split('`');
    let mut in_code = false;
    for part in parts.by_ref() {
        if in_code {
            res.push_str("<code>");
            res.push_str(part);
            res.push_str("</code>");
        } else {
            let mut bold_res = String::new();
            let mut bold_parts = part.split("**");
            let mut in_bold = false;
            for bpart in bold_parts.by_ref() {
                if in_bold {
                    bold_res.push_str("<strong>");
                    bold_res.push_str(bpart);
                    bold_res.push_str("</strong>");
                } else {
                    bold_res.push_str(bpart);
                }
                in_bold = !in_bold;
            }
            res.push_str(&bold_res);
        }
        in_code = !in_code;
    }
    res
}

fn inline_markdown_to_html(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;

    while !rest.is_empty() {
        if let Some(start) = rest.find("[[") {
            let before = &rest[..start];
            out.push_str(&process_basic_inlines(before));
            let after_open = &rest[start + 2..];
            if let Some(end) = after_open.find("]]") {
                let inside = &after_open[..end];
                let (target, label) = match inside.split_once('|') {
                    Some((t, l)) => (t.trim(), l.trim()),
                    None => (inside.trim(), inside.trim()),
                };
                let href = format!("./{}", page_id_to_filename(target));
                out.push_str(&format!(
                    r#"<a class="wiki-link" href="{href}">{}</a>"#,
                    escape_html(label)
                ));
                rest = &after_open[end + 2..];
            } else {
                out.push_str("[[");
                rest = after_open;
            }
        } else {
            out.push_str(&process_basic_inlines(rest));
            break;
        }
    }
    out
}

pub fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_list = false;

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                if in_list {
                    html.push_str("</ul>\n");
                    in_list = false;
                }
                let lang = trimmed.trim_start_matches('`').trim();
                let class_attr = if lang.is_empty() {
                    String::new()
                } else {
                    format!(r#" class="language-{}""#, escape_html(lang))
                };
                html.push_str(&format!("<pre><code{class_attr}>"));
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }

        if let Some(h) = trimmed.strip_prefix("# ") {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<h1>{}</h1>\n", inline_markdown_to_html(h)));
        } else if let Some(h) = trimmed.strip_prefix("## ") {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<h2>{}</h2>\n", inline_markdown_to_html(h)));
        } else if let Some(h) = trimmed.strip_prefix("### ") {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<h3>{}</h3>\n", inline_markdown_to_html(h)));
        } else if let Some(h) = trimmed.strip_prefix("#### ") {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<h4>{}</h4>\n", inline_markdown_to_html(h)));
        } else if let Some(item) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
        {
            if !in_list {
                html.push_str("<ul>\n");
                in_list = true;
            }
            html.push_str(&format!("  <li>{}</li>\n", inline_markdown_to_html(item)));
        } else if let Some(quote) = trimmed.strip_prefix("> ") {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!(
                "<blockquote>{}</blockquote>\n",
                inline_markdown_to_html(quote)
            ));
        } else if trimmed.is_empty() {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
        } else {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<p>{}</p>\n", inline_markdown_to_html(trimmed)));
        }
    }

    if in_code_block {
        html.push_str("</code></pre>\n");
    }
    if in_list {
        html.push_str("</ul>\n");
    }

    html
}

fn site_css() -> &'static str {
    r#"/* WikiCraft High-Contrast WCAG 7:1 Static Site Stylesheet */
:root {
  --bg: #090d16;
  --panel: #131c2e;
  --card: #182238;
  --border: #273852;
  --text: #f1f5f9;
  --text-muted: #94a3b8;
  --accent: #38bdf8;
  --accent-hover: #0284c7;
  --link: #60a5fa;
  --code-bg: #0b1120;
  --badge-green: #10b981;
  --badge-blue: #0ea5e9;
  --badge-red: #f43f5e;
  --badge-gray: #64748b;
  --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  --font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

[data-theme="light"] {
  --bg: #f8fafc;
  --panel: #ffffff;
  --card: #f1f5f9;
  --border: #cbd5e1;
  --text: #0f172a;
  --text-muted: #475569;
  --accent: #0284c7;
  --accent-hover: #0369a1;
  --link: #2563eb;
  --code-bg: #e2e8f0;
}

* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  font-family: var(--font-sans);
  background: var(--bg);
  color: var(--text);
  line-height: 1.6;
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

header {
  background: var(--panel);
  border-bottom: 1px solid var(--border);
  padding: 0.75rem 1.5rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: sticky;
  top: 0;
  z-index: 50;
}

header .brand {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-weight: 700;
  font-size: 1.15rem;
  color: var(--accent);
  text-decoration: none;
}

header .actions {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.theme-btn {
  background: var(--card);
  border: 1px solid var(--border);
  color: var(--text);
  padding: 0.4rem 0.8rem;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
  font-weight: 600;
}

.theme-btn:hover {
  border-color: var(--accent);
}

.container {
  display: flex;
  flex: 1;
}

aside.sidebar {
  width: 320px;
  background: var(--panel);
  border-right: 1px solid var(--border);
  padding: 1.25rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  height: calc(100vh - 58px);
  position: sticky;
  top: 58px;
  overflow-y: auto;
}

.sidebar input#search-input {
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--card);
  color: var(--text);
  font-size: 0.9rem;
}

.sidebar input#search-input:focus {
  outline: none;
  border-color: var(--accent);
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.sidebar-nav a {
  display: block;
  padding: 0.45rem 0.65rem;
  color: var(--text-muted);
  text-decoration: none;
  border-radius: 6px;
  font-size: 0.9rem;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-nav a:hover, .sidebar-nav a.active {
  background: var(--card);
  color: var(--accent);
}

main.content {
  flex: 1;
  padding: 2.5rem 3.5rem;
  max-width: 960px;
  margin: 0 auto;
}

.breadcrumbs {
  font-size: 0.85rem;
  color: var(--text-muted);
  margin-bottom: 1.5rem;
}

.breadcrumbs a {
  color: var(--link);
  text-decoration: none;
}

h1 { font-size: 2.2rem; margin-bottom: 1rem; color: var(--text); border-bottom: 2px solid var(--border); padding-bottom: 0.5rem; }
h2 { font-size: 1.5rem; margin-top: 1.75rem; margin-bottom: 0.75rem; color: var(--text); }
h3 { font-size: 1.2rem; margin-top: 1.25rem; margin-bottom: 0.5rem; color: var(--text); }
p { margin-bottom: 1rem; }
ul, ol { margin-left: 1.5rem; margin-bottom: 1rem; }
li { margin-bottom: 0.25rem; }
blockquote {
  border-left: 4px solid var(--accent);
  padding: 0.5rem 1rem;
  background: var(--card);
  color: var(--text-muted);
  margin: 1rem 0;
  border-radius: 0 6px 6px 0;
}

pre {
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1rem;
  overflow-x: auto;
  margin-bottom: 1.5rem;
}

code {
  font-family: var(--font-mono);
  font-size: 0.9em;
  background: var(--code-bg);
  padding: 0.15rem 0.35rem;
  border-radius: 4px;
  border: 1px solid var(--border);
}

pre code {
  background: transparent;
  padding: 0;
  border: none;
}

a {
  color: var(--link);
  text-decoration: underline;
}

a.wiki-link {
  color: var(--accent);
  font-weight: 600;
  text-decoration: none;
  background: rgba(56, 189, 248, 0.1);
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  border: 1px solid rgba(56, 189, 248, 0.25);
}

a.wiki-link:hover {
  background: rgba(56, 189, 248, 0.2);
}

.backlinks-box {
  margin-top: 3rem;
  padding: 1.25rem;
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 8px;
}

.backlinks-box h3 {
  margin-top: 0;
  font-size: 1rem;
  color: var(--text-muted);
}

.badge {
  display: inline-block;
  padding: 0.2rem 0.55rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
}

.badge-accepted { background: var(--badge-green); color: #ffffff; }
.badge-proposed { background: var(--badge-blue); color: #ffffff; }
.badge-rejected { background: var(--badge-red); color: #ffffff; }
.badge-superseded { background: var(--badge-gray); color: #ffffff; }

.adr-card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 1.25rem;
  margin-bottom: 1.25rem;
}

.adr-card .header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.75rem;
}

@media (max-width: 768px) {
  .container { flex-direction: column; }
  aside.sidebar { width: 100%; height: auto; position: static; }
  main.content { padding: 1.5rem 1rem; }
}
"#
}

fn site_js() -> &'static str {
    r#"document.addEventListener('DOMContentLoaded', () => {
  const q = document.getElementById('search-input');
  if (q) {
    q.addEventListener('input', (e) => {
      const val = e.target.value.toLowerCase();
      document.querySelectorAll('.sidebar-nav a').forEach(el => {
        const text = el.textContent.toLowerCase();
        el.style.display = text.includes(val) ? '' : 'none';
      });
    });
  }
  const btn = document.getElementById('theme-toggle');
  if (btn) {
    btn.addEventListener('click', () => {
      const cur = document.documentElement.getAttribute('data-theme') || 'dark';
      const next = cur === 'dark' ? 'light' : 'dark';
      document.documentElement.setAttribute('data-theme', next);
      localStorage.setItem('wikicraft-theme', next);
      btn.textContent = next === 'dark' ? '☀️ Light' : '🌙 Dark';
    });
    const saved = localStorage.getItem('wikicraft-theme');
    if (saved) {
      document.documentElement.setAttribute('data-theme', saved);
      btn.textContent = saved === 'dark' ? '☀️ Light' : '🌙 Dark';
    }
  }
});
"#
}

fn render_html_page(
    title: &str,
    current_filename: &str,
    content_html: &str,
    pages: &[wikicraft_store::PageSummary],
    backlinks: &[String],
) -> String {
    let mut nav_links = String::new();
    nav_links.push_str(r#"<a href="./index.html" class=""#);
    if current_filename == "index.html" {
        nav_links.push_str("active");
    }
    nav_links.push_str(r#"">🏠 Home / Portal</a>"#);

    nav_links.push_str(r#"<a href="./adrs.html" class=""#);
    if current_filename == "adrs.html" {
        nav_links.push_str("active");
    }
    nav_links.push_str(r#"">⚖️ ADR Matrix</a>"#);

    for p in pages {
        let fn_name = page_id_to_filename(&p.id);
        let active = if current_filename == fn_name {
            " active"
        } else {
            ""
        };
        nav_links.push_str(&format!(
            r#"<a href="./{fn_name}" class="{active}" title="{}">📄 {}</a>"#,
            escape_html(&p.title),
            escape_html(&p.id)
        ));
    }

    let mut bl_section = String::new();
    if !backlinks.is_empty() {
        bl_section.push_str(r#"<div class="backlinks-box"><h3>🔗 Backlinks</h3><ul>"#);
        for bl in backlinks {
            let fn_name = page_id_to_filename(bl);
            bl_section.push_str(&format!(
                r#"<li><a class="wiki-link" href="./{fn_name}">{}</a></li>"#,
                escape_html(bl)
            ));
        }
        bl_section.push_str("</ul></div>");
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="dark">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{} - WikiCraft</title>
  <link rel="stylesheet" href="./style.css">
</head>
<body>
  <header>
    <a href="./index.html" class="brand">
      <span>📚</span>
      <span>WikiCraft</span>
    </a>
    <div class="actions">
      <a href="./adrs.html" style="font-size:0.9rem; text-decoration:none; color:var(--text-muted); font-weight:600;">ADRs</a>
      <button id="theme-toggle" class="theme-btn">☀️ Light</button>
    </div>
  </header>
  <div class="container">
    <aside class="sidebar">
      <input type="text" id="search-input" placeholder="🔍 Filter pages...">
      <nav class="sidebar-nav">
        {}
      </nav>
    </aside>
    <main class="content">
      <div class="breadcrumbs">
        <a href="./index.html">WikiCraft</a> / {}
      </div>
      {}
      {}
    </main>
  </div>
  <script src="./script.js"></script>
</body>
</html>"#,
        escape_html(title),
        nav_links,
        escape_html(title),
        content_html,
        bl_section
    )
}

/// Export wiki documentation and ADRs to a standalone static HTML website.
pub fn export_static_site(out_dir: &Path, ctx: &mut Ctx) -> Result<Value, String> {
    std::fs::create_dir_all(out_dir)
        .map_err(|e| format!("failed to create out_dir {}: {e}", out_dir.display()))?;

    // Write CSS and JS
    std::fs::write(out_dir.join("style.css"), site_css())
        .map_err(|e| format!("failed to write style.css: {e}"))?;
    std::fs::write(out_dir.join("script.js"), site_js())
        .map_err(|e| format!("failed to write script.js: {e}"))?;

    let pages = ctx.store.list_pages().map_err(|e| e.to_string())?;
    let adrs = ctx.store.list_adrs().map_err(|e| e.to_string())?;

    // Export each page
    let mut exported_pages = 0usize;
    for p_summary in &pages {
        if let Ok(full_page) = ctx.store.get_page(&p_summary.id) {
            let backlinks = ctx.store.get_backlinks(&p_summary.id).unwrap_or_default();
            let body_html = markdown_to_html(&full_page.content);
            let fn_name = page_id_to_filename(&p_summary.id);
            let full_html =
                render_html_page(&full_page.title, &fn_name, &body_html, &pages, &backlinks);
            let target_path = out_dir.join(fn_name);
            std::fs::write(&target_path, full_html)
                .map_err(|e| format!("failed to write {}: {e}", target_path.display()))?;
            exported_pages += 1;
        }
    }

    // Export adrs.html
    let mut adrs_content = String::new();
    adrs_content.push_str("<h1>Architecture Decision Records (ADRs)</h1>\n");
    adrs_content.push_str("<p>Overview of technical architecture and architectural decisions recorded in this system.</p>\n");

    for adr in &adrs {
        let badge_class = match adr.status.to_lowercase().as_str() {
            "accepted" => "badge-accepted",
            "proposed" => "badge-proposed",
            "rejected" => "badge-rejected",
            _ => "badge-superseded",
        };
        adrs_content.push_str(&format!(
            r#"<div class="adr-card">
  <div class="header">
    <h2>{} - {}</h2>
    <span class="badge {badge_class}">{}</span>
  </div>
  <p><strong>Context:</strong> {}</p>
  <p><strong>Decision:</strong> {}</p>
  <p><strong>Consequences:</strong> {}</p>
"#,
            escape_html(&adr.id),
            escape_html(&adr.title),
            escape_html(&adr.status),
            escape_html(&adr.context),
            escape_html(&adr.decision),
            escape_html(&adr.consequences),
        ));
        if let Some(ref sup) = adr.superseded_by {
            adrs_content.push_str(&format!(
                r#"  <p><strong>Superseded By:</strong> {}</p>"#,
                escape_html(sup)
            ));
        }
        adrs_content.push_str("</div>\n");
    }

    let adrs_html = render_html_page(
        "Architecture Decision Records",
        "adrs.html",
        &adrs_content,
        &pages,
        &[],
    );
    std::fs::write(out_dir.join("adrs.html"), adrs_html)
        .map_err(|e| format!("failed to write adrs.html: {e}"))?;

    // Export index.html
    let mut index_content = String::new();
    index_content.push_str("<h1>WikiCraft Documentation Hub</h1>\n");
    index_content.push_str(&format!(
        "<p>Welcome to the static documentation portal. Currently tracking <strong>{}</strong> documentation pages and <strong>{}</strong> architecture decisions.</p>\n",
        pages.len(),
        adrs.len()
    ));

    // If there is a "Home" or "overview" page, show its content as the home article!
    let featured = pages
        .iter()
        .find(|p| p.id.eq_ignore_ascii_case("home") || p.id.ends_with("/overview"))
        .or_else(|| pages.first());

    if let Some(f) = featured {
        if let Ok(feat_page) = ctx.store.get_page(&f.id) {
            index_content.push_str(&format!(
                "<hr style=\"border:0; border-top:1px solid var(--border); margin:1.5rem 0;\">\n<h2>Featured: {}</h2>\n",
                escape_html(&feat_page.title)
            ));
            index_content.push_str(&markdown_to_html(&feat_page.content));
        }
    }

    let index_html = render_html_page("Home", "index.html", &index_content, &pages, &[]);
    std::fs::write(out_dir.join("index.html"), index_html)
        .map_err(|e| format!("failed to write index.html: {e}"))?;

    Ok(json!({
        "exported_pages": exported_pages,
        "exported_adrs": adrs.len(),
        "out_dir": out_dir.to_string_lossy(),
        "index_file": out_dir.join("index.html").to_string_lossy()
    }))
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
        Command::SeedDemo => {
            let mut pages_created = 0usize;
            let mut adrs_created = 0usize;

            let p1 = ("architecture/overview", "Architecture Overview", "# WikiCraft Documentation Hub\n\nWelcome to **WikiCraft**, the local-first documentation, SQLite FTS5 search, and Architecture Decision Record (ADR) studio.\n\n## Core Subsystems\n- [[architecture/storage]]: Storage layer architecture, SQLite WAL, and schema design.\n- [[architecture/search]]: FTS5 full-text indexing, tokenizer, and query capabilities.\n- [[fleet/standards]]: Engineering standards, crash guarantees, and protocol conventions.\n- [[guides/getting-started]]: Guide for creating pages, linking documents, and recording decisions.\n\n## Architecture Decisions\nSee [[ADR-001]] for our initial decision on SQLite WAL and FTS5, and [[ADR-002]] for wiki-link bidirectional graph indexing.");

            let p2 = ("architecture/storage", "Storage Layer Architecture", "# Storage Layer & WAL Design\n\nWikiCraft persists all pages, backlinks, and ADRs in a local SQLite database.\n\n## Key Properties\n- **Write-Ahead Logging (WAL)**: Concurrent reads during writes, sub-millisecond transactions.\n- **Embedded Simplicity**: Zero network dependencies, zero background daemons required.\n- **Cross-Links**: Automatically parsed from `[[page-slug]]` syntax.\n\nReferenced by [[architecture/overview]].");

            let p3 = ("architecture/search", "FTS5 Search Engine", "# SQLite FTS5 Search Engine\n\nWikiCraft maintains a synchronous `pages_fts` virtual table powered by SQLite FTS5.\n\n## Features\n- Sub-millisecond prefix and phrase searches.\n- Automated snippet extraction highlighting matched terms.\n- Zero external search service requirements.\n\nSee [[architecture/overview]] and [[architecture/storage]].");

            let p4 = ("fleet/standards", "Fleet Engineering Standards", "# Fleet Engineering Standards\n\nAll tools in the fleet follow unified protocols:\n1. **Zero-Crash Guarantee**: `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` enforced.\n2. **High-Contrast Theming**: Minimum 7:1 WCAG contrast on all text surfaces.\n3. **Event Bus Protocol**: Status changes publish signals across the fleet via `craft-biz` bus.\n\nSee [[architecture/overview]] and [[guides/getting-started]].");

            let p5 = ("guides/getting-started", "Getting Started with WikiCraft", "# Getting Started Guide\n\n1. Browse documentation in the **Pages** tree.\n2. Search across all articles instantly in the **Search** tab.\n3. Inspect decision lifecycles in the **ADR Matrix** tab.\n4. Explore bidirectional cross-references in the **Wiki Graph** tab.\n\nBack to [[architecture/overview]].");

            for (id, title, content) in &[p1, p2, p3, p4, p5] {
                if ctx.store.save_page(id, title, content).is_ok() {
                    let links = extract_wiki_links(content);
                    let _ = ctx.store.save_links(id, &links);
                    pages_created += 1;
                }
            }

            let a1 = ("ADR-001", "Adopt SQLite WAL and FTS5 for Local Knowledge Store", AdrStatus::Accepted, "Need a zero-dependency, local-first embedded database for documentation and search without external services.", "Adopt SQLite with WAL journaling and FTS5 full-text indexing.", "Instant startup, zero cloud cost, ACID reliability, and sub-millisecond search.", None);
            let a2 = (
                "ADR-002",
                "Bidirectional Wiki-Link Syntax Parsing",
                AdrStatus::Accepted,
                "Need cross-document linking and backlink tracking across markdown pages.",
                "Parse [[page-slug]] syntax at save time and maintain a normalized links table.",
                "Instant backlink lookup and topology graph visualization.",
                None,
            );
            let a3 = (
                "ADR-003",
                "Centralized Remote Cloud Wiki Sync",
                AdrStatus::Rejected,
                "Proposal to sync all markdown documents to a remote cloud SaaS wiki.",
                "Rejected in favor of local-first SQLite WAL and fleet event bus synchronization.",
                "Preserves zero-latency, full privacy, and offline functionality.",
                None,
            );

            for (id, title, status, context, decision, consequences, superseded_by) in &[a1, a2, a3]
            {
                if ctx
                    .store
                    .save_adr(
                        id,
                        title,
                        status,
                        context,
                        decision,
                        consequences,
                        *superseded_by,
                    )
                    .is_ok()
                {
                    adrs_created += 1;
                }
            }

            Ok(json!({
                "seeded": true,
                "pages_created": pages_created,
                "adrs_created": adrs_created
            }))
        }
        Command::GhWikiPull { repo } => gh_wiki_pull(repo, ctx),
        Command::GhWikiPush { repo, message } => gh_wiki_push(repo, message.as_deref(), ctx),
        Command::GhWikiStatus { repo } => gh_wiki_status(repo),
        Command::SiteExport { out_dir } => export_static_site(out_dir, ctx),
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
        tool(
            "wiki_page_list",
            "List all documentation pages",
            json!({}),
            &[],
        ),
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
        tool(
            "wiki_adr_list",
            "List all Architecture Decision Records (ADRs)",
            json!({}),
            &[],
        ),
        tool(
            "wiki_tree_import",
            "Recursively import a directory of Markdown documentation files into the wiki",
            json!({
                "path": { "type": "string" }
            }),
            &["path"],
        ),
        tool(
            "wiki_seed_demo",
            "Seed the wiki with sample architecture documentation, cross-links, and ADRs",
            json!({}),
            &[],
        ),
        tool(
            "wiki_gh_pull",
            "Clone or pull a GitHub repository wiki and ingest all markdown pages into the SQLite index",
            json!({
                "repo": { "type": "string", "description": "GitHub repository name, e.g. 'sandraschi/wikicraft' or 'wikicraft'" }
            }),
            &["repo"],
        ),
        tool(
            "wiki_gh_push",
            "Export all wiki pages and ADRs to a GitHub Wiki repository, generate _Sidebar.md navigation, and push via git",
            json!({
                "repo": { "type": "string", "description": "GitHub repository name, e.g. 'sandraschi/wikicraft' or 'wikicraft'" },
                "message": { "type": "string", "description": "Optional git commit message" }
            }),
            &["repo"],
        ),
        tool(
            "wiki_gh_status",
            "Check local clone state and remote commit information for a GitHub Wiki repository",
            json!({
                "repo": { "type": "string", "description": "GitHub repository name, e.g. 'sandraschi/wikicraft' or 'wikicraft'" }
            }),
            &["repo"],
        ),
        tool(
            "wiki_site_export",
            "Export the wiki documentation and ADRs to a standalone static HTML website ready for GitHub Pages or offline browsing",
            json!({
                "out_dir": { "type": "string", "description": "Target output directory for the static HTML site" }
            }),
            &["out_dir"],
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
        let temp_dir =
            std::env::temp_dir().join(format!("wiki_core_test_{}", uuid::Uuid::new_v4()));
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

        let export_dir = temp_dir.join("site");
        let export_cmd = Command::SiteExport {
            out_dir: export_dir.clone(),
        };
        let export_res = dispatch(&export_cmd, &mut ctx).unwrap();
        assert_eq!(
            export_res.get("exported_pages").unwrap().as_u64().unwrap(),
            1
        );
        assert_eq!(
            export_res.get("exported_adrs").unwrap().as_u64().unwrap(),
            1
        );
        assert!(export_dir.join("index.html").exists());
        assert!(export_dir.join("adrs.html").exists());
        assert!(export_dir.join("style.css").exists());
    }

    #[test]
    fn test_markdown_to_html_formatting() {
        let md = "# Title\n\nWelcome to [[guide/getting-started|Getting Started]].\n\n```rust\nfn main() {}\n```";
        let html = markdown_to_html(md);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains(
            r#"<a class="wiki-link" href="./guide__getting-started.html">Getting Started</a>"#
        ));
        assert!(html.contains(r#"<pre><code class="language-rust">fn main() {}"#));
    }
}
