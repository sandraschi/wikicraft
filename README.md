# wikicraft

> Local-first Markdown documentation, SQLite FTS5 search, and Architecture Decision Record (ADR) lifecycle studio.

Part of the **Back-Office Craft Suite** (`craft-biz`). Replaces Confluence (\$60/seat/mo), Notion Team (\$100/seat/mo), and Slite.

## Features

- **Local-first SQLite WAL:** Zero-cloud latency, ACID reliability, instant startup.
- **SQLite FTS5 Full-Text Search:** Instant sub-millisecond search across thousands of markdown documents with highlighted snippets.
- **ADR Lifecycle State Machine:** Formal tracking of architectural decisions across `proposed`, `accepted`, `rejected`, and `superseded` states.
- **Wiki-Link Topology Graph:** Parses `[[page-slug]]` syntax and tracks bi-directional links and backlinks.
- **Dual Surface:** Stdio JSON-RPC MCP server for autonomous AI coding agents + native `egui 0.37` desktop application.
- **Fleet Bus Integration:** Automatically publishes `adr.accepted` and `adr.superseded` bus events across fleet applications.

## Quick Start

```bash
# Check status
wikicraft status

# Save a page
wikicraft page-save architecture/db "Database Architecture" "We use SQLite WAL with [[architecture/overview]]."

# Search documentation
wikicraft search "SQLite"

# Propose an ADR
wikicraft adr-save ADR-001 "Adopt Rust 2021" proposed "Need memory safety" "Adopt Rust across fleet" "Zero runtime overhead"

# Launch MCP server
wikicraft mcp

# Launch native desktop workstation
wikicraft-app
```

## Ports

- Frontend Webapp: `11260`
- Backend / MCP API: `11261`
