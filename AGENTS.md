# WikiCraft — Agent Protocol and Architecture

## Overview
`wikicraft` is the documentation and architecture decision repository for the Back-Office Craft Suite.

## Workspace Layout
- `crates/wikicraft-store`: SQLite WAL schema (`pages`, `adrs`, `links`, and FTS5 index `pages_fts`).
- `crates/wikicraft-core`: Core command dispatch, markdown scanner, wiki-link graph, ADR state transitions, and 12 MCP tools.
- `crates/wikicraft-mcp`: Single-binary CLI and stdio JSON-RPC MCP server.
- `crates/wikicraft-app`: Native `egui 0.37` desktop workstation with documentation tree, ADR matrix, and full-text search.

## Standards
- Enforce `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
- Zero-crash guarantee across CLI and MCP stdio channels.
- Use `craft-biz` bus for fleet event notifications (`adr.accepted`).
