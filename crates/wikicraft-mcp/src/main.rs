//! `wikicraft`: CLI and MCP stdio server in one binary.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clap::{Parser, Subcommand};
use craft_biz::bus::Bus;
use craft_biz::mcp::{serve_stdio, ToolHost};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use wikicraft_core::{dispatch, tools, Command, Confinement, Ctx, APP, VERSION};
use wikicraft_store::Store;

#[derive(Parser)]
#[command(
    name = "wikicraft",
    version,
    about = "wikicraft: local-first markdown wiki, FTS5 search, and ADR studio. CLI + MCP."
)]
struct Cli {
    /// SQLite database. Default: $WIKICRAFT_DB or %LOCALAPPDATA%\wikicraft\wikicraft.db
    #[arg(long, global = true)]
    db: Option<PathBuf>,
    /// Print machine-readable JSON only.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Wiki status and counts.
    Status,
    /// Save or update a markdown page.
    PageSave {
        id: String,
        title: String,
        content: String,
    },
    /// Get a page by ID with backlinks.
    PageGet { id: String },
    /// List all pages.
    PageList,
    /// Delete a page by ID.
    PageDelete { id: String },
    /// Search pages using SQLite FTS5.
    Search { query: String },
    /// Find pages linking to an ID.
    Backlinks { id: String },
    /// Save or update an Architecture Decision Record.
    AdrSave {
        id: String,
        title: String,
        #[arg(default_value = "proposed")]
        status: String,
        #[arg(default_value = "")]
        context: String,
        #[arg(default_value = "")]
        decision: String,
        #[arg(default_value = "")]
        consequences: String,
        #[arg(long)]
        superseded_by: Option<String>,
    },
    /// Get an ADR by ID.
    AdrGet { id: String },
    /// List all ADRs.
    AdrList,
    /// Recursively import a directory of markdown files.
    TreeImport { path: PathBuf },
    /// Seed the wiki with sample documentation and ADRs.
    SeedDemo,
    /// Pull and ingest a GitHub repository wiki.
    GhPull {
        /// Repository name (e.g. 'sandraschi/wikicraft' or 'wikicraft')
        repo: String,
    },
    /// Export and push wiki pages to a GitHub repository wiki.
    GhPush {
        /// Repository name (e.g. 'sandraschi/wikicraft' or 'wikicraft')
        repo: String,
        /// Optional commit message
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Inspect local clone state and remote commit for a GitHub repository wiki.
    GhStatus {
        /// Repository name (e.g. 'sandraschi/wikicraft' or 'wikicraft')
        repo: String,
    },
    /// Export wiki and ADRs to a standalone static HTML website.
    SiteExport {
        /// Target output directory
        out_dir: PathBuf,
    },
    /// Run HTTP/SSE gateway and MCP JSON-RPC server on port 11261.
    Serve {
        /// Port to listen on (default 11261)
        #[arg(short, long, default_value_t = 11261)]
        port: u16,
        /// Host address to bind
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Watch a directory of markdown files and auto-ingest updates into SQLite.
    Watch {
        /// Directory path to watch
        path: PathBuf,
        /// Polling interval in seconds
        #[arg(short, long, default_value_t = 2)]
        poll_interval_secs: u64,
    },
    /// Drain incoming bus signals.
    InboxDrain,
    /// Print tool specifications.
    Tools {
        #[arg(long)]
        markdown: bool,
    },
    /// Run as MCP stdio server.
    Mcp,
}

struct Host {
    ctx: Ctx,
}

impl ToolHost for Host {
    fn server_name(&self) -> &str {
        APP
    }
    fn server_version(&self) -> &str {
        VERSION
    }
    fn tools(&self) -> Vec<Value> {
        tools()
    }
    fn call(&mut self, name: &str, args: &Value) -> Result<Value, String> {
        let cmd = Command::from_tool(name, args)?;
        dispatch(&cmd, &mut self.ctx)
    }
}

fn default_db() -> PathBuf {
    if let Ok(p) = std::env::var("WIKICRAFT_DB") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join("wikicraft").join("wikicraft.db")
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let db_path = cli.db.unwrap_or_else(default_db);

    let is_mcp = matches!(cli.cmd, Some(Cmd::Mcp));
    if !is_mcp {
        if let Some(parent) = db_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    let store = match Store::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening database at {}: {e}", db_path.display());
            return ExitCode::FAILURE;
        }
    };

    let actor = if is_mcp {
        "agent".to_string()
    } else {
        std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "cli".into())
    };

    let ctx = Ctx {
        store,
        confinement: Confinement(None),
        bus: Bus::new(Bus::default_root()),
        actor,
    };

    if is_mcp {
        let mut host = Host { ctx };
        return match serve_stdio(&mut host) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("wikicraft mcp: {e}");
                ExitCode::FAILURE
            }
        };
    }

    match cli.cmd.unwrap_or(Cmd::Status) {
        Cmd::Mcp => unreachable!(),
        Cmd::Serve { port, host } => serve_http(&host, port, ctx),
        Cmd::Watch {
            path,
            poll_interval_secs,
        } => run_watcher(&path, poll_interval_secs, ctx),
        Cmd::Tools { markdown } => {
            if markdown {
                for t in tools() {
                    let name = t.get("name").and_then(Value::as_str).unwrap_or("");
                    let desc = t.get("description").and_then(Value::as_str).unwrap_or("");
                    println!("- `{name}`: {desc}");
                }
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tools()).unwrap_or_default()
                );
            }
            ExitCode::SUCCESS
        }
        cmd => {
            let mut ctx = ctx;
            let cmd_target = match cmd {
                Cmd::Mcp | Cmd::Serve { .. } | Cmd::Watch { .. } | Cmd::Tools { .. } => {
                    unreachable!()
                }
                Cmd::Status => Command::Status,
                Cmd::SiteExport { out_dir } => Command::SiteExport { out_dir },
                Cmd::PageSave { id, title, content } => Command::PageSave { id, title, content },
                Cmd::PageGet { id } => Command::PageGet { id },
                Cmd::PageList => Command::PageList,
                Cmd::PageDelete { id } => Command::PageDelete { id },
                Cmd::Search { query } => Command::PageSearch { query },
                Cmd::Backlinks { id } => Command::PageBacklinks { id },
                Cmd::AdrSave {
                    id,
                    title,
                    status,
                    context,
                    decision,
                    consequences,
                    superseded_by,
                } => Command::AdrSave {
                    id,
                    title,
                    status,
                    context,
                    decision,
                    consequences,
                    superseded_by,
                },
                Cmd::AdrGet { id } => Command::AdrGet { id },
                Cmd::AdrList => Command::AdrList,
                Cmd::TreeImport { path } => Command::TreeImport { path },
                Cmd::SeedDemo => Command::SeedDemo,
                Cmd::GhPull { repo } => Command::GhWikiPull { repo },
                Cmd::GhPush { repo, message } => Command::GhWikiPush { repo, message },
                Cmd::GhStatus { repo } => Command::GhWikiStatus { repo },
                Cmd::InboxDrain => Command::InboxDrain,
            };

            match dispatch(&cmd_target, &mut ctx) {
                Ok(val) => {
                    println!("{}", serde_json::to_string_pretty(&val).unwrap_or_default());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

fn urlencoding_decode(val: &str) -> String {
    let mut result = String::new();
    let bytes = val.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex_str) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte_val) = u8::from_str_radix(hex_str, 16) {
                    result.push(byte_val as char);
                    i += 3;
                    continue;
                }
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn handle_mcp_jsonrpc(body: &[u8], ctx_lock: &Arc<Mutex<Ctx>>) -> (u16, Value) {
    let req: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => {
            return (
                400,
                json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": format!("Parse error: {e}") }
                }),
            )
        }
    };

    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = match req.get("method").and_then(Value::as_str) {
        Some(m) => m,
        None => {
            return (
                400,
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32600, "message": "Missing method" }
                }),
            )
        }
    };

    match method {
        "initialize" => (
            200,
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": APP,
                        "version": VERSION
                    }
                }
            }),
        ),
        "notifications/initialized" => (
            200,
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            }),
        ),
        "ping" => (
            200,
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            }),
        ),
        "tools/list" => (
            200,
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": tools()
                }
            }),
        ),
        "tools/call" => {
            let params = match req.get("params") {
                Some(p) => p,
                None => {
                    return (
                        400,
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32602, "message": "Missing params for tools/call" }
                        }),
                    )
                }
            };
            let tool_name = match params.get("name").and_then(Value::as_str) {
                Some(n) => n,
                None => {
                    return (
                        400,
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32602, "message": "Missing tool name in params" }
                        }),
                    )
                }
            };
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let cmd = match Command::from_tool(tool_name, &args) {
                Ok(c) => c,
                Err(e) => {
                    return (
                        200,
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32602, "message": e }
                        }),
                    )
                }
            };

            let mut ctx = match ctx_lock.lock() {
                Ok(c) => c,
                Err(e) => {
                    return (
                        500,
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32603, "message": format!("Lock error: {e}") }
                        }),
                    )
                }
            };

            match dispatch(&cmd, &mut ctx) {
                Ok(res) => (
                    200,
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [
                                {
                                    "type": "text",
                                    "text": serde_json::to_string_pretty(&res).unwrap_or_default()
                                }
                            ]
                        }
                    }),
                ),
                Err(e) => (
                    200,
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "isError": true,
                            "content": [
                                {
                                    "type": "text",
                                    "text": format!("Error executing tool {tool_name}: {e}")
                                }
                            ]
                        }
                    }),
                ),
            }
        }
        other => (
            404,
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Method not found: {other}") }
            }),
        ),
    }
}

fn handle_http_client(mut stream: TcpStream, port: u16, ctx_lock: &Arc<Mutex<Ctx>>) {
    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });

    let mut line = String::new();
    if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
        return;
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0].to_string();
    let uri = parts[1].to_string();

    let mut content_length = 0usize;
    loop {
        line.clear();
        if reader.read_line(&mut line).is_err() || line == "\r\n" || line == "\n" || line.is_empty()
        {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                if let Ok(len) = v.trim().parse::<usize>() {
                    content_length = len;
                }
            }
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        let _ = reader.read_exact(&mut body);
    }

    if method.eq_ignore_ascii_case("OPTIONS") {
        let resp = "HTTP/1.1 204 No Content\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
Connection: close\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    let (status_code, body_json) =
        if method.eq_ignore_ascii_case("GET") && (uri == "/health" || uri == "/api/health") {
            (
                200,
                json!({
                    "status": "ok",
                    "app": APP,
                    "version": VERSION,
                    "port": port
                }),
            )
        } else if method.eq_ignore_ascii_case("GET") && uri == "/api/status" {
            if let Ok(mut ctx) = ctx_lock.lock() {
                match dispatch(&Command::Status, &mut ctx) {
                    Ok(v) => (200, v),
                    Err(e) => (500, json!({ "error": e })),
                }
            } else {
                (500, json!({ "error": "lock contention" }))
            }
        } else if method.eq_ignore_ascii_case("GET") && uri == "/api/pages" {
            if let Ok(mut ctx) = ctx_lock.lock() {
                match dispatch(&Command::PageList, &mut ctx) {
                    Ok(v) => (200, v),
                    Err(e) => (500, json!({ "error": e })),
                }
            } else {
                (500, json!({ "error": "lock contention" }))
            }
        } else if method.eq_ignore_ascii_case("GET") && uri == "/api/adrs" {
            if let Ok(mut ctx) = ctx_lock.lock() {
                match dispatch(&Command::AdrList, &mut ctx) {
                    Ok(v) => (200, v),
                    Err(e) => (500, json!({ "error": e })),
                }
            } else {
                (500, json!({ "error": "lock contention" }))
            }
        } else if method.eq_ignore_ascii_case("GET") && uri.starts_with("/api/search") {
            let query = if let Some((_, q_str)) = uri.split_once('?') {
                let mut q_val = String::new();
                for param in q_str.split('&') {
                    if let Some((k, v)) = param.split_once('=') {
                        if k == "q" || k == "query" {
                            q_val = urlencoding_decode(&v.replace('+', " "));
                        }
                    }
                }
                q_val
            } else {
                String::new()
            };

            if let Ok(mut ctx) = ctx_lock.lock() {
                match dispatch(&Command::PageSearch { query }, &mut ctx) {
                    Ok(v) => (200, v),
                    Err(e) => (500, json!({ "error": e })),
                }
            } else {
                (500, json!({ "error": "lock contention" }))
            }
        } else if method.eq_ignore_ascii_case("POST") && (uri == "/mcp" || uri == "/api/mcp") {
            handle_mcp_jsonrpc(&body, ctx_lock)
        } else {
            (
                404,
                json!({ "error": "not found", "uri": uri, "method": method }),
            )
        };

    let body_bytes = serde_json::to_vec_pretty(&body_json).unwrap_or_default();
    let status_line = match status_code {
        200 => "HTTP/1.1 200 OK",
        400 => "HTTP/1.1 400 Bad Request",
        404 => "HTTP/1.1 404 Not Found",
        _ => "HTTP/1.1 500 Internal Server Error",
    };

    let headers = format!(
        "{status_line}\r\n\
Content-Type: application/json; charset=utf-8\r\n\
Content-Length: {}\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
Connection: close\r\n\r\n",
        body_bytes.len()
    );

    let _ = stream.write_all(headers.as_bytes());
    let _ = stream.write_all(&body_bytes);
}

fn serve_http(host: &str, port: u16, ctx: Ctx) -> ExitCode {
    let addr = format!("{host}:{port}");
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {addr}: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("=== WikiCraft HTTP/MCP Dual Server ===");
    println!("Listening on:       http://{addr}");
    println!("Health endpoint:    http://{addr}/health");
    println!("API endpoints:      http://{addr}/api/status");
    println!("                    http://{addr}/api/pages");
    println!("                    http://{addr}/api/adrs");
    println!("                    http://{addr}/api/search?q=...");
    println!("MCP JSON-RPC:       POST http://{addr}/mcp");
    println!("CORS:               Enabled (*)");
    println!("Press Ctrl+C to stop.");

    let ctx_lock = Arc::new(Mutex::new(ctx));

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let lock_clone = Arc::clone(&ctx_lock);
                std::thread::spawn(move || {
                    handle_http_client(s, port, &lock_clone);
                });
            }
            Err(e) => {
                eprintln!("Incoming connection error: {e}");
            }
        }
    }

    ExitCode::SUCCESS
}

fn scan_md_files(root: &std::path::Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    scan_md_files(&p, out);
                }
            } else if p.is_file() {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    if ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown") {
                        out.push(p);
                    }
                }
            }
        }
    }
}

fn run_watcher(dir: &std::path::Path, poll_interval_secs: u64, mut ctx: Ctx) -> ExitCode {
    if !dir.exists() {
        eprintln!(
            "Error: Watch target directory does not exist: {}",
            dir.display()
        );
        return ExitCode::FAILURE;
    }
    println!("=== WikiCraft Live Watcher ===");
    println!("Watching directory: {}", dir.display());
    println!("Polling interval:   {poll_interval_secs}s");
    println!("Database:           Active SQLite FTS5 store");
    println!("Press Ctrl+C to stop.");

    use std::collections::HashMap;
    use std::time::SystemTime;

    let mut mtimes: HashMap<PathBuf, SystemTime> = HashMap::new();

    loop {
        let mut current_files = Vec::new();
        scan_md_files(dir, &mut current_files);

        for file in current_files {
            let mtime = match std::fs::metadata(&file).and_then(|m| m.modified()) {
                Ok(t) => t,
                Err(_) => continue,
            };

            let needs_ingest = match mtimes.get(&file) {
                Some(&prev) => mtime > prev,
                None => true,
            };

            if needs_ingest {
                if let Ok(content) = std::fs::read_to_string(&file) {
                    let rel = file
                        .strip_prefix(dir)
                        .unwrap_or(&file)
                        .to_string_lossy()
                        .replace('\\', "/");
                    let title = file
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(&rel)
                        .to_string();

                    if ctx.store.save_page(&rel, &title, &content).is_ok() {
                        let links = wikicraft_core::extract_wiki_links(&content);
                        let _ = ctx.store.save_links(&rel, &links);
                        let _ = ctx.bus.emit(
                            "wikicraft",
                            "*",
                            "wiki.page.updated",
                            json!({
                                "id": rel,
                                "title": title,
                                "bytes": content.len()
                            }),
                        );
                        println!(
                            "[watch] Ingested: {} ({} bytes, {} links)",
                            rel,
                            content.len(),
                            links.len()
                        );
                    }
                }
                mtimes.insert(file, mtime);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(poll_interval_secs));
    }
}
