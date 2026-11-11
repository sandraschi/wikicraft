# WikiCraft MCP & Transport Reference

WikiCraft exposes 17 Model Context Protocol (MCP) tools over stdio JSON-RPC, HTTP JSON-RPC gateway on port 11261, and CLI.

## MCP Tools (17 Tools)

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
| `wiki_gh_pull` | `repo` | Clones or pulls a GitHub repository wiki (`owner/repo`) and ingests pages into SQLite. |
| `wiki_gh_push` | `repo`, `message` (opt) | Exports pages and ADRs, auto-generates `_Sidebar.md`, and pushes via git to GitHub Wiki. |
| `wiki_gh_status` | `repo` | Inspects local clone state, remote wiki URL, and last commit for a GitHub Wiki. |
| `wiki_site_export` | `out_dir` | Exports documentation and ADRs to a standalone static HTML website ready for GitHub Pages (`gh-pages`). |
| `wiki_inbox_drain` | none | Acknowledges and clears pending signals in the wiki event bus inbox. |

## Dual Transport (HTTP + MCP on port 11261)

Run the server with:
```powershell
wikicraft-mcp serve --port 11261
```

Available HTTP endpoints:
- `GET /health`: Health probe reporting status, app name, version, and active port.
- `GET /api/status`: Store statistics (page count, ADR count, link count).
- `GET /api/pages`: List all documentation pages.
- `GET /api/adrs`: List all Architecture Decision Records.
- `GET /api/search?q=<query>`: SQLite FTS5 full-text search.
- `POST /mcp`: JSON-RPC 2.0 MCP endpoint (`initialize`, `tools/list`, `tools/call`, `ping`).
- CORS headers (`Access-Control-Allow-Origin: *`) are active for all endpoints.

## Live Filesystem Watcher

Monitor a directory for real-time changes and auto-ingest modified or new markdown files:
```powershell
wikicraft-mcp watch "D:\Dev\repos\mcp-central-docs\standards" --poll-interval-secs 2
```

## Static Site Export

Export the entire knowledge store to a standalone static HTML website with responsive high-contrast styling and instant client-side search:
```powershell
wikicraft-mcp site-export "./dist/site"
```
