## WikiCraft v0.3.1

- Accepting an ADR emits one bus event. The demo seed is marked as demo.

### Highlights
- **High-Contrast Design (`craft-biz-ui::shell`)**: High-contrast WCAG 7:1 theme tokens, Light/Dark mode switcher, persistent settings in `%LOCALAPPDATA%\wikicraft\settings.json`.
- **Interactive Markdown Reader & Editor**: Dual-mode structured markdown viewer with section headers, code blocks, bullet formatting, and in-place `✏️ Edit Page` / `💾 Save` mode.
- **Clickable Wiki Navigation**: Interactive navigation for `[[wiki-links]]`, bidirectional backlinks, SQLite FTS5 search results, and node graph.
- **Documentation Ingestion Drawer (`📂 Ingest Docs`)**: 1-click batch ingestion of `mcp-central-docs` (`standards`, `patterns`, `operations`, `troubleshooting`, `architecture`) and custom local directory scanning.
- **GitHub Wiki Synchronization (`🌐 GH Wiki`)**: Full bidirectional support for GitHub wiki repositories (`.wiki.git`) with automatic navigation sidebar (`_Sidebar.md`) and footer (`_Footer.md`) generation (`wiki_gh_pull`, `wiki_gh_push`, `wiki_gh_status`).
- **Static Site / HTML Export (`🚀 Export Site`)**: Standalone, static HTML website generator (`wiki_site_export`) with zero external CDN dependencies, dark/light theme toggle, real-time client search filter, and ADR matrix ready for GitHub Pages (`gh-pages`).
- **Dual Transport Gateway (HTTP + MCP on port 11261)**: Concurrent HTTP server and JSON-RPC MCP gateway (`wikicraft-mcp serve --port 11261`) with `/health`, `/api/status`, `/api/pages`, `/api/adrs`, `/api/search?q=...`, and `/mcp` endpoints.
- **Live Filesystem Watcher**: Background directory watcher (`wikicraft-mcp watch <dir>`) auto-ingesting modified and newly created markdown files in real-time.
- **Distribution Package**: Anthropic-standard `.mcpb` bundle (`wikicraft-v0.2.0.mcpb`) via `@anthropic-ai/mcpb` with full tool definitions and automated `scripts/release.ps1` release pipeline.
