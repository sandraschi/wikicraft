//! `wikicraft`: CLI and MCP stdio server in one binary.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use clap::{Parser, Subcommand};
use craft_biz::bus::Bus;
use craft_biz::mcp::{serve_stdio, ToolHost};
use serde_json::Value;
use std::path::PathBuf;
use std::process::ExitCode;
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

    let mut ctx = ctx;

    let cmd_target = match cli.cmd.unwrap_or(Cmd::Status) {
        Cmd::Mcp => unreachable!(),
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
            return ExitCode::SUCCESS;
        }
        Cmd::Status => Command::Status,
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
