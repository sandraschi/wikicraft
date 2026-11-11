# wikicraft

> Local-first Markdown documentation, SQLite FTS5 full-text search, and Architecture Decision Record (ADR) lifecycle studio.

Part of the **Back-Office Craft Suite** (`craft-biz`). Replaces Confluence (\$60/seat/mo), Notion Team (\$100/seat/mo), and Slite with zero-cloud latency, complete privacy, and air-gapped reliability.

## Features

- **Local-First SQLite WAL:** Zero-cloud latency, ACID reliability, embedded single-file storage in `%LOCALAPPDATA%\wikicraft\wikicraft.db`.
- **SQLite FTS5 Full-Text Search:** Instant sub-millisecond search across thousands of markdown documents with highlighted snippets.
- **ADR Lifecycle State Machine:** Formal governance of architectural decisions across `proposed`, `accepted`, `rejected`, and `superseded` states.
- **Wiki-Link Topology Graph:** Parses `[[page-slug]]` and `[[target|label]]` syntax; tracks bidirectional outgoing edges and incoming backlinks.
- **High-Contrast Desktop Workstation (`egui 0.37`):** Built with `craft-biz-ui` high-contrast tokens (WCAG 7:1 compliant), Dark/Light theme switching, structured markdown reader, live page editor, and sidebar search filter.
- **1-Click Fleet Documentation Ingestion:** Ingest curated `mcp-central-docs` sections (`Standards`, `Patterns`, `Operations`, `Troubleshooting`, `Architecture`) or any local folder into SQLite WAL + FTS5 in seconds.
- **Bootstrap Seed:** One-click `🌱 Seed Demo` button to populate sample architecture docs and ADRs instantly.
- **Dual Surface:** Native desktop app for human operators + stdio JSON-RPC MCP server with 13 tools for autonomous AI coding agents.
- **Fleet Bus Integration:** Automatically publishes `adr.accepted` bus signals across fleet applications via `craft-biz` bus.

## Quick Start

### 1. Launch Desktop Workstation

```powershell
.\start.ps1        # or .\start.bat
```

### 2. Command Line Interface (CLI)

```bash
# Seed initial demo documentation and ADRs
wikicraft seed-demo

# Check status and document counts
wikicraft status

# Save or update a markdown page
wikicraft page-save architecture/db "Database Architecture" "We use SQLite WAL with [[architecture/overview]]."

# Search documentation with highlighted snippets
wikicraft search "SQLite"

# Propose an Architecture Decision Record (ADR)
wikicraft adr-save ADR-001 "Adopt Rust 2021" proposed "Need memory safety" "Adopt Rust across fleet" "Zero runtime overhead"

# Import an entire folder of markdown documents
wikicraft tree-import d:/Dev/repos/mcp-central-docs/standards

# Launch MCP server on stdio for AI coding agents
wikicraft mcp
```

## MCP Server Tools (13)

When launched with `wikicraft mcp`, the server exposes 13 tools over JSON-RPC stdio:

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
| `wiki_inbox_drain` | none | Acknowledges and clears pending signals in the wiki event bus inbox. |
