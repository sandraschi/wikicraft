# WikiCraft — Tiered Execution Plan

**Parent Standard:** [`RUST_ARTCRAFT_STANDARD.md`](../../mcp-central-docs/standards/RUST_ARTCRAFT_STANDARD.md)  
**Menu Bar Standard:** [`CRAFT_MENU_BAR_STANDARD.md`](../../mcp-central-docs/standards/rust-artcraft/CRAFT_MENU_BAR_STANDARD.md)  
**Architecture Model:** 4-Tier ArtCraft Incremental Delivery  
**Status:** Active  

---

## Architecture Flow

```
Tier 1: Core Engine ──> Tier 2: MCP / CLI ──> Tier 3: Desktop UI ──> Tier 4: Menu Bar, Help & Packaging
(wikicraft-core)       (wikicraft-mcp)       (wikicraft-app)       (Hardened Workstation)
```

---

## Tier Specifications

### Tier 1 — Core Markdown & Link Graph Engine (`wikicraft-core`, `wikicraft-store`)
- [x] L1 `Command::dispatch` universal wiki execution bus.
- [x] SQLite WAL database with FTS5 sub-millisecond full-text search.
- [x] Local markdown tree crawler and wiki-link (`[[target]]`) parser.
- [x] Bidirectional link graph topology and backlink indexer.
- [x] Architecture Decision Record (ADR) lifecycle manager (`proposed`, `accepted`, `rejected`, `superseded`).
- [x] Cross-craft event emission (`adr.accepted` -> fleet bus).
- [x] `#![forbid(unsafe_code)]` and zero-panic clippy lints.

### Tier 2 — Headless Driver & FastMCP (`wikicraft-mcp`)
- [x] Unified CLI `wikicraft-cli` with `--json` output.
- [x] FastMCP stdio server communicating over JSON-RPC 2.0.
- [x] GitHub Wiki bi-directional sync driver.
- [x] Headless `--smoke` verification without window initialization.

### Tier 3 — Desktop Workstation UI (`wikicraft-app`)
- [x] Immediate-mode `egui` 0.37 / `wgpu` workstation (`craft-biz-ui`).
- [x] Space and page tree navigation with live markdown document viewer.
- [x] Split-pane editor, incoming backlinks drawer, and ADR registry view.

### Tier 4 — Craft Menu Bar Standard, Help Suite & Hardened Verification
- [ ] Universal 8-menu bar (`File`, `Edit`, `View`, `Insert`, `Modify`, `Commands`, `Window`, `Help`) with `switch_on_hover`.
- [ ] Mandatory 5-dialog Help suite (`Command Palette...`, `Keyboard Shortcuts...`, `Project Files & Docs...`, `Licenses & Notices...`, `About WikiCraft`).
- [ ] Fleet UI/UX Pattern 2 adherence: viewport-centric navigation, top button row mapped to View toggles, panel visibility managed via `View` / `Window`.
- [ ] Tofu Guard verification (100% font glyph coverage, zero tofu placeholder boxes).
- [ ] Headless `--smoke` parity validation.
- [ ] Standalone release binary packaging and NSIS installer.
