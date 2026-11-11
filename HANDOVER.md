# WikiCraft - Handover Document

**Last Updated**: 2026-11-11  
**Current State**: v0.2.0 Tier 1 + L3 Desktop Workstation (Store + Core + CLI + MCP + egui 0.37 App operational)  
**Primary Artifact**: [sandraschi/wikicraft](https://github.com/sandraschi/wikicraft) | `D:\Dev\repos\wikicraft`  

---

## 1. What This Replaces

WikiCraft replaces cloud-hosted team wikis and knowledge bases for sovereign engineering fleets:
- **Replaces**: Confluence (\$60/seat/mo), Notion Team (\$100/seat/mo), and Slite (\$10/seat/mo).
- **Why**:
  - **Zero-Cloud & Zero Latency**: Single SQLite file with WAL journal (`wikicraft.db`), zero cloud vendor subscription fees, zero telemetry leaks, instant startup.
  - **SQLite FTS5 Full-Text Search**: Sub-millisecond snippet queries across thousands of markdown documents without external search services.
  - **ADR Lifecycle Governance**: Formally tracks architectural decisions across `proposed`, `accepted`, `rejected`, and `superseded` states, broadcasting `adr.accepted` bus signals across fleet apps via `craft-biz` bus.
  - **Bidirectional Wiki Graph**: Parses `[[page-slug]]` syntax, tracking outgoing edges and incoming backlinks.
  - **GitHub Wiki Sync**: Native clone, pull, and push bridge with `https://github.com/<owner>/<repo>.wiki.git`, auto-generating `_Sidebar.md` navigation.
  - **Dual Surfaces**: 60fps native desktop workstation (`wikicraft.exe`) and stdio JSON-RPC MCP server (`wikicraft-mcp.exe` with 16 tools) sharing one core.

---

## 2. What Was Done

- **High-Contrast Theming**: Integrated `craft-biz-ui::shell` and `theme` tokens conforming to WCAG 7:1 contrast ratios. Added Dark / Light mode toggle persisting to `ui.json`.
- **Interactive Markdown Reader & Editor**:
  - Structured rendering for `#`/`##`/`###` headings, monospace code panels with `t.sunken` background, and indented bullet lists.
  - Replaced grayed-out non-interactive widget with read-only viewer + inline `✏️ Edit Page` / `💾 Save` mode.
  - Clickable wiki-links (`[[target]]`), backlinks, and search matches that immediately navigate between pages.
- **Documentation Ingestion & Seeding**:
  - **`🌱 Seed Demo`**: One-click bootstrap creating 5 architecture documents and 3 ADRs.
  - **`📂 Ingest Docs`**: One-click batch ingestion of `mcp-central-docs` sections (`Standards`, `Patterns`, `Operations`, `Troubleshooting`, `Architecture`) and arbitrary local folders.
  - Real-time `🔍 Filter pages...` search bar on the sidebar for navigating hundreds of indexed pages.
- **GitHub Wiki Synchronization**:
  - Implemented `wiki_gh_pull`, `wiki_gh_push`, and `wiki_gh_status` in core, CLI, MCP, and desktop UI.
  - Automated `_Sidebar.md` and `_Footer.md` generation on push.
  - Default repository auto-resolution to `sandraschi/<repo>`.
- **Fleet Packaging & Release**:
  - Aligned binary names per `RELEASE_FACTORY.md`: `wikicraft.exe` (desktop workstation) and `wikicraft-mcp.exe` (CLI/server).
  - Shipped and published release `v0.1.0` and `v0.2.0` (GitHub Wiki sync, 16 MCP tools) with plain unversioned executables, craft-launcher portable zip, and `SHA256SUMS.txt`.
- **Quality & Standards**:
  - Zero compiler warnings, zero clippy warnings across workspace with `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
  - All unit tests passing, automated smoke check `--smoke` operational.

---

## 3. Deliberately Not Done (Boundaries & Honest Gaps)

- **No hosted web server**: WikiCraft is purely native desktop (`egui`) and stdio JSON-RPC MCP server; no web ports are opened.
- **No cloud vendor database**: Strictly local-first SQLite WAL; no cloud sync dependencies.
- **No WYSIWYG editor**: Preserves plain Markdown syntax integrity; no proprietary rich-text bloat.
- **No background file-watcher daemon**: Ingestion is triggered explicitly (via UI button, CLI, or MCP tool) rather than continuous file system polling to prevent CPU and disk thrashing.

---

## 4. Immediate Next Steps

1. Mirror `HANDOVER.md` to `mcp-central-docs/projects/wikicraft/HANDOVER.md`.
2. Configure `wikicraft-mcp` in fleet agent tool configurations (`claude_desktop_config.json`, opencode) for centralized documentation querying.
3. Add export format options (e.g. static HTML or mdBook generation) for publishing to GitHub Pages or static doc sites.
