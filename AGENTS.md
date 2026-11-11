# WikiCraft — Agent Protocol and Architecture

## Overview
`wikicraft` is the documentation, knowledge base, and architecture decision repository for the Back-Office Craft Suite (`craft-biz`). It operates local-first on SQLite WAL with synchronous FTS5 indexing, fleet bus integration, and bidirectional GitHub Wiki synchronization.

## Workspace Layout
- `crates/wikicraft-store`: SQLite WAL schema (`pages`, `adrs`, `links`, and FTS5 index `pages_fts`).
- `crates/wikicraft-core`: Core command dispatch, markdown scanner, wiki-link graph, GitHub Wiki git synchronizer, ADR state transitions, and 16 MCP tools.
- `crates/wikicraft-mcp`: Single-binary CLI (`wikicraft-mcp`) and stdio JSON-RPC MCP server for autonomous agents.
- `crates/wikicraft-app`: Native `egui 0.37` desktop workstation (`wikicraft`) with high-contrast theming (`craft-biz-ui`), documentation tree filter, structured markdown viewer, page editor, ADR matrix, topology graph, and GitHub Wiki sync drawer.

## MCP Tools (16)
- `wiki_status`: Health check, page/ADR/link counts.
- `wiki_page_save`: Store page content, parse `[[wiki-links]]`.
- `wiki_page_get`: Retrieve page, backlinks, and outgoing links.
- `wiki_page_list`: Enumerate all pages.
- `wiki_page_delete`: Delete page by ID.
- `wiki_page_search`: Query SQLite FTS5 index with highlighted snippets.
- `wiki_page_backlinks`: Query incoming links for an ID.
- `wiki_adr_save`: Create or advance an ADR (emits `adr.accepted` bus event).
- `wiki_adr_get`: Fetch an ADR by ID.
- `wiki_adr_list`: Enumerate all ADRs.
- `wiki_tree_import`: Crawl a directory and index all `.md` files.
- `wiki_seed_demo`: Seed initial documentation structure and ADRs.
- `wiki_gh_pull`: Pull and ingest a GitHub repository wiki (`owner/repo`).
- `wiki_gh_push`: Export pages/ADRs, generate `_Sidebar.md`, and push to GitHub Wiki.
- `wiki_gh_status`: Inspect clone status and remote commit for a GitHub Wiki.
- `wiki_inbox_drain`: Acknowledge and clear bus inbox signals.

## Standards
- Enforce `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- Zero-crash guarantee across CLI and MCP stdio channels.
- Use `craft-biz` bus for fleet event notifications (`adr.accepted`).
