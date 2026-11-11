# WikiCraft - Handover Document

**Last Updated**: 2026-11-11  
**Current State**: v0.3.0 Next Tier Suite Complete (Store + Core + CLI + Dual Transport MCP [stdio + HTTP port 11261] + egui 0.37 Desktop Studio + Static Site Generator + Live Watcher + .mcpb bundle)  
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
  - **Static Site / HTML Export**: Generates standalone static HTML websites ready for GitHub Pages (`gh-pages`) with responsive WCAG 7:1 dark/light stylesheet, client search filter, and ADR matrix.
  - **Dual Transport Gateway**: Concurrent HTTP server and JSON-RPC MCP gateway on port `11261` (`/health`, `/api/status`, `/api/pages`, `/api/adrs`, `/api/search?q=...`, `/mcp`).
  - **Live Filesystem Watcher**: Background directory watcher (`wikicraft-mcp watch <dir>`) auto-ingesting modified and newly created markdown files in real-time.
  - **Triple Surface**: Native desktop studio (`wikicraft.exe`), stdio/HTTP MCP server (`wikicraft-mcp.exe` with 17 tools), and Anthropic `.mcpb` bundle (`wikicraft-v0.2.0.mcpb`).

---

## 2. What Was Done

- **High-Contrast Theming**: Integrated `craft-biz-ui::shell` and `theme` tokens conforming to WCAG 7:1 contrast ratios. Added Dark / Light mode toggle persisting to `ui.json`.
- **Interactive Markdown Reader & Editor**:
  - Structured rendering for `#`/`##`/`###` headings, monospace code panels with `t.sunken` background, and indented bullet lists.
  - Dual-mode structured viewer + inline `✏️ Edit Page` / `💾 Save` mode.
  - Clickable wiki-links (`[[target]]`), backlinks, and search matches that immediately navigate between pages.
- **Documentation Ingestion & Seeding**:
  - **`🌱 Seed Demo`**: One-click bootstrap creating 5 architecture documents and 3 ADRs.
  - **`📂 Ingest Docs`**: One-click batch ingestion of `mcp-central-docs` sections (`Standards`, `Patterns`, `Operations`, `Troubleshooting`, `Architecture`) and arbitrary local folders (ingested 581 pages in seconds).
  - Real-time `🔍 Filter pages...` search bar on the sidebar for navigating hundreds of indexed pages.
- **GitHub Wiki Synchronization**:
  - Implemented `wiki_gh_pull`, `wiki_gh_push`, and `wiki_gh_status` in core, CLI, MCP, and desktop UI.
  - Automated `_Sidebar.md` and `_Footer.md` generation on push.
  - Default repository auto-resolution to `sandraschi/<repo>`.
- **Static Site / HTML Generator**:
  - Implemented `wiki_site_export` / `wikicraft-mcp site-export <out_dir>` and UI button `🚀 Export Site`.
  - Generates self-contained static HTML site with `index.html`, `adrs.html`, `<slug>.html`, `style.css` (WCAG 7:1 theme), and `script.js` (instant client-side search filter + theme toggle).
- **Dual Transport Gateway (HTTP + MCP on port 11261)**:
  - Implemented `wikicraft-mcp serve --port 11261`.
  - Exposes `GET /health`, `GET /api/status`, `GET /api/pages`, `GET /api/adrs`, `GET /api/search?q=...`, and `POST /mcp` (JSON-RPC 2.0 MCP handler) with full CORS support.
- **Live Filesystem Watcher**:
  - Implemented `wikicraft-mcp watch <dir> [--poll-interval-secs 2]` with mtime change detection, auto-ingestion into SQLite WAL, and `wiki.page.updated` bus signal emission.
- **Distribution Package & Release Pipeline**:
  - Built Anthropic-standard `mcpb/manifest.json` validated via `@anthropic-ai/mcpb`.
  - Built `scripts/release.ps1` automated release pipeline (Phase 0 preflight, Phase 1 gates, Phase 2 release build & smoke tests, Phase 3 artifact staging [.exe, portable zip, .mcpb bundle, SHA256SUMS.txt], Phase 4 publish with `gh`).
- **Quality & Standards**:
  - Zero compiler warnings, zero clippy warnings across workspace with `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
  - All unit tests passing, automated smoke checks `--smoke` operational.

---

## 3. Deliberately Not Done (Boundaries & Honest Gaps)

- **No cloud vendor database**: Strictly local-first SQLite WAL; no cloud database dependencies.
- **No WYSIWYG editor**: Preserves plain Markdown syntax integrity; no proprietary rich-text bloat.
- **GitHub Wiki first-page creation**: As documented, GitHub requires creating the initial wiki page via web UI before the git backend activates for clone/push.

---

## 4. Immediate Next Steps

1. Mirror `HANDOVER.md` to `mcp-central-docs/projects/wikicraft/HANDOVER.md`.
2. Commit and push the full next-tier suite to GitHub.
3. Run `scripts/release.ps1` to publish artifacts to GitHub Releases.
