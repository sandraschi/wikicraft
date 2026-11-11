# WikiCraft - Agent Protocol and Architecture

## Overview
`wikicraft` is the documentation, knowledge base, and architecture decision repository for the Back-Office Craft Suite (`craft-biz`). It operates local-first on SQLite WAL with synchronous FTS5 indexing.

## Workspace Layout
- `crates/wikicraft-store`: SQLite WAL schema (`pages`, `adrs`, `links`, and FTS5 index `pages_fts`).
- `crates/wikicraft-core`: Core command dispatch, markdown scanner, wiki-link graph, ADR state transitions, and 13 MCP tools.
- `crates/wikicraft-mcp`: Single-binary CLI and stdio JSON-RPC MCP server for autonomous agents.
- `crates/wikicraft-app`: Native `egui 0.37` desktop workstation with high-contrast theming (`craft-biz-ui`), documentation tree filter, structured markdown viewer, page editor, ADR matrix, and topology graph.

## MCP Tools (13)
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
- `wiki_inbox_drain`: Acknowledge and clear bus inbox signals.

## Standards
- Enforce `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- Zero-crash guarantee across CLI and MCP stdio channels.
- Use `craft-biz` bus for fleet event notifications (`adr.accepted`).
