# WikiCraft MCP Tools Reference

WikiCraft exposes 13 Model Context Protocol (MCP) tools over stdio JSON-RPC.

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
| `wiki_inbox_drain` | none | Acknowledges and clears pending signals in the wiki event bus inbox. |
