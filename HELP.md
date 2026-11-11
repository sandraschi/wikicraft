# WikiCraft Quick Reference

## Desktop Workstation (`wikicraft.exe`)

Launch via:
```powershell
.\start.ps1        # or .\start.bat
# or directly:
target\release\wikicraft.exe
```

### Key UI Features
- **High-Contrast Theming:** Dark and Light mode toggle in the top-right header with WCAG 7:1 compliant color tokens.
- **Documentation Tree & Filter:** Browse pages by ID/title with a real-time `🔍 Filter pages...` search bar.
- **Interactive Markdown Reader:** Structured rendering for `#`/`##`/`###` headings, monospace code blocks, and indented lists.
- **Clickable Wiki-Links & Backlinks:** Clicking any `[[target]]` link or backlink pill instantly navigates to that page.
- **Page Editor:** Toggle `✏️ Edit Page` to modify titles and markdown content with `💾 Save`.
- **🌱 Seed Demo:** Bootstrap sample architecture docs, wiki links, and ADRs with a single click.
- **📂 Ingest Docs:** 1-click batch ingestion of `mcp-central-docs` sections (`Standards`, `Patterns`, `Operations`, `Troubleshooting`, `Architecture`) or any custom markdown directory.
- **🌐 GH Wiki Sync:** 1-click bidirectional synchronization drawer for any `sandraschi/<repo>` GitHub Wiki (Pull, Push with auto-generated `_Sidebar.md`, Status).
- **🏛️ ADR Matrix:** Filterable table tracking architectural decisions across `proposed`, `accepted`, `rejected`, and `superseded` states.
- **🕸️ Wiki Graph:** Interactive topology view mapping outgoing links and incoming backlinks.

---

## CLI & MCP Server (`wikicraft-mcp.exe`)

```bash
# General status and counts
wikicraft-mcp status

# Markdown Pages
wikicraft-mcp page-save <id> <title> <content>
wikicraft-mcp page-get <id>
wikicraft-mcp page-list
wikicraft-mcp page-delete <id>
wikicraft-mcp search <query>
wikicraft-mcp backlinks <id>

# Architecture Decision Records (ADRs)
wikicraft-mcp adr-save <id> <title> [status] [context] [decision] [consequences] [--superseded-by <id>]
wikicraft-mcp adr-get <id>
wikicraft-mcp adr-list

# Directory Import & Ingestion
wikicraft-mcp tree-import <directory-path>

# GitHub Wiki Synchronization
wikicraft-mcp gh-pull <repo>
wikicraft-mcp gh-push <repo> [--message <msg>]
wikicraft-mcp gh-status <repo>

# Bootstrap Demo Data
wikicraft-mcp seed-demo

# Event Bus Inbox
wikicraft-mcp inbox-drain

# Run as Stdio MCP Server for AI Coding Agents
wikicraft-mcp mcp

# Inspect Tool Specifications
wikicraft-mcp tools [--markdown]
```
