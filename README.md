# wikicraft

> Local-first Markdown documentation, SQLite FTS5 full-text search, GitHub Wiki synchronizer, and Architecture Decision Record (ADR) lifecycle studio.

Part of the **Back-Office Craft Suite** (`craft-biz`). Replaces Confluence (\$60/seat/mo), Notion Team (\$100/seat/mo), and Slite with zero-cloud latency, complete privacy, bidirectional GitHub Wiki synchronization, and air-gapped reliability.

## Features

- **Local-First SQLite WAL:** Zero-cloud latency, ACID reliability, embedded single-file storage in `%LOCALAPPDATA%\wikicraft\wikicraft.db`.
- **SQLite FTS5 Full-Text Search:** Instant sub-millisecond search across thousands of markdown documents with highlighted snippets.
- **GitHub Wiki Bidirectional Sync:** Direct clone, pull, and push integration with any `https://github.com/<owner>/<repo>.wiki.git` repository, with automated `_Sidebar.md` navigation generation and `Home.md` mapping.
- **ADR Lifecycle State Machine:** Formal governance of architectural decisions across `proposed`, `accepted`, `rejected`, and `superseded` states.
- **Wiki-Link Topology Graph:** Parses `[[page-slug]]` and `[[target|label]]` syntax; tracks bidirectional outgoing edges and incoming backlinks.
- **High-Contrast Desktop Workstation (`egui 0.37`):** Built with `craft-biz-ui` high-contrast tokens (WCAG 7:1 compliant), Dark/Light theme switching, structured markdown reader, live page editor, and sidebar search filter.
- **1-Click Fleet Documentation Ingestion:** Ingest curated `mcp-central-docs` sections (`Standards`, `Patterns`, `Operations`, `Troubleshooting`, `Architecture`) or any local folder into SQLite WAL + FTS5 in seconds.
- **Bootstrap Seed:** One-click `🌱 Seed Demo` button to populate sample architecture docs and ADRs instantly.
- **Dual Surface:** Native desktop app (`wikicraft`) for human operators + stdio & HTTP JSON-RPC MCP server (`wikicraft-mcp`) with 17 tools for autonomous AI coding agents.
- **Static Site Export (`site-export`):** Generates standalone static HTML websites ready for GitHub Pages (`gh-pages`) with responsive WCAG 7:1 dark/light stylesheet, client search filter, and ADR matrix.
- **Dual Transport Gateway (HTTP + MCP on port 11261):** Concurrent HTTP server with `/health`, `/api/status`, `/api/pages`, `/api/adrs`, `/api/search?q=...`, and `/mcp` endpoints.
- **Live Filesystem Watcher:** Background directory watcher (`wikicraft-mcp watch <dir>`) auto-ingesting modified and newly created markdown files in real-time.
- **Anthropic `.mcpb` Bundle:** Fully validated `@anthropic-ai/mcpb` distribution package with automated release pipeline (`scripts/release.ps1`).
- **Fleet Bus Integration:** Automatically publishes `adr.accepted` and `wiki.page.updated` bus signals across fleet applications via `craft-biz` bus.

## Quick Start

### 1. Launch Desktop Workstation

```powershell
.\start.ps1        # or .\start.bat
# or directly:
target\release\wikicraft.exe
```

### 2. Command Line Interface (CLI) & MCP Server

```bash
# Seed initial demo documentation and ADRs
wikicraft-mcp seed-demo

# Check status and document counts
wikicraft-mcp status

# Synchronize with a GitHub repository wiki
wikicraft-mcp gh-pull sandraschi/wikicraft
wikicraft-mcp gh-status sandraschi/wikicraft
wikicraft-mcp gh-push sandraschi/wikicraft --message "docs: sync from WikiCraft"

# Export standalone static HTML website
wikicraft-mcp site-export ./dist/site

# Run concurrent HTTP/MCP server on port 11261
wikicraft-mcp serve --port 11261

# Watch a directory for real-time markdown updates
wikicraft-mcp watch d:/Dev/repos/mcp-central-docs/standards

# Save or update a markdown page
wikicraft-mcp page-save architecture/db "Database Architecture" "We use SQLite WAL with [[architecture/overview]]."

# Search documentation with highlighted snippets
wikicraft-mcp search "SQLite"

# Propose an Architecture Decision Record (ADR)
wikicraft-mcp adr-save ADR-001 "Adopt Rust 2021" proposed "Need memory safety" "Adopt Rust across fleet" "Zero runtime overhead"

# Import an entire folder of markdown documents
wikicraft-mcp tree-import d:/Dev/repos/mcp-central-docs/standards

# Launch MCP server on stdio for AI coding agents
wikicraft-mcp mcp
```

## MCP Server Tools (17)

When launched with `wikicraft-mcp mcp`, the server exposes 16 tools over JSON-RPC stdio:

| Tool | Parameters | Description |
|---|---|---|
| `wiki_status` | none | Reports database health, page counts, ADR counts, and link counts. |
| `wiki_page_save` | `id`, `title`, `content` | Saves or updates a markdown documentation page and extracts `[[wiki-links]]`. |
| `wiki_page_get` | `id` | Retrieves a page by ID with backlinks and outgoing links. |
| `wiki_page_list` | none | Lists all documentation pages with version numbers and modification dates. |
| `wiki_page_delete` | `id` | Deletes a documentation page by ID. |
| `wiki_page_search` | `query` | Runs SQLite FTS5 full-text search and returns highlighted snippet matches. |
| `wiki_page_backlinks` | `id` | Discovers all pages that link to the specified page ID. |
| `wiki_adr_save` | `id`, `title`, `status`, `context`, `decision`, `consequences`, `superseded_by` | Creates or updates an ADR with status transitions and emits bus events. |
| `wiki_adr_get` | `id` | Fetches details and history of an Architecture Decision Record. |
| `wiki_adr_list` | none | Lists all Architecture Decision Records. |
| `wiki_tree_import` | `path` | Recursively indexes a folder of markdown files into SQLite WAL + FTS5. |
| `wiki_seed_demo` | none | Seeds the database with initial architecture docs, cross-links, and sample ADRs. |
| `wiki_gh_pull` | `repo` | Clones or pulls a GitHub repository wiki (`owner/repo`) and ingests pages into SQLite. |
| `wiki_gh_push` | `repo`, `message` (opt) | Exports pages and ADRs, auto-generates `_Sidebar.md`, and pushes via git to GitHub Wiki. |
| `wiki_gh_status` | `repo` | Inspects local clone state, remote wiki URL, and last commit for a GitHub Wiki. |
| `wiki_inbox_drain` | none | Acknowledges and clears pending signals in the wiki event bus inbox. |
