# WikiCraft: Product Requirements Document

## 1. Executive Summary

`wikicraft` is the central documentation, knowledge repository, and Architecture Decision Record (ADR) management system of the Back-Office Craft Suite (`craft-biz`). It provides a local-first, zero-cloud alternative to enterprise wiki platforms with sub-millisecond full-text search and bidirectional link topology tracking.

## 2. Key Capabilities

- **Local Markdown Tree Ingestion:** Automatically crawls local repositories, directories, or curated fleet documentation trees (`mcp-central-docs`), extracting wiki links (`[[target]]`) and indexing markdown into SQLite WAL and FTS5.
- **Fast Full-Text Search (FTS5):** Sub-millisecond snippet queries without cloud dependencies, latency, or telemetry leaks.
- **ADR Governance & Event Bus:** Formally tracks architectural decisions across lifecycle states (`proposed`, `accepted`, `rejected`, `superseded`) and automatically publishes `adr.accepted` bus events across fleet applications via `craft-biz`.
- **Bidirectional Wiki-Link Graph:** Automatically parses `[[page-slug]]` / `[[target|label]]` syntax and computes outgoing edges and incoming backlinks.
- **GitHub Wiki Synchronization:** Bi-directional synchronization with GitHub Wiki repositories (`<repo>.wiki.git`), with automated `_Sidebar.md` table of contents generation and `Home.md` integration.
- **High-Contrast Desktop Workstation:** Native `egui 0.37` desktop application with high-contrast WCAG 7:1 theming (`craft-biz-ui`), light/dark mode switching, live document viewer with formatted headers/code blocks, and inline page editor.
- **Instant Bootstrapping:** One-click `🌱 Seed Demo` and `📂 Ingest Docs` (MCD Standards, Patterns, Operations, Troubleshooting, Architecture) for zero-friction setup.
- **Zero-Cloud Air-Gapped Operation:** Eliminates external vendor subscription fees and preserves confidential system design data.

## 3. Replaces & Cost Savings

- **Replaces:** Confluence (\$60/seat/mo), Notion Team (\$100/seat/mo), Slite (\$10/seat/mo).
- **Cost Reduction:** 100% elimination of recurring cloud subscription fees, zero latency, and guaranteed availability during network partitioning or offline field work.
