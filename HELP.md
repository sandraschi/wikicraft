# WikiCraft Quick Reference

## Desktop Workstation (`wikicraft-app`)

Launch via:
```powershell
.\start.ps1        # or .\start.bat
```

### Key UI Features
- **High-Contrast Theming:** Dark and Light mode toggle in the top-right header with WCAG 7:1 compliant color tokens.
- **Documentation Tree & Filter:** Browse pages by ID/title with a real-time `🔍 Filter pages...` search bar.
- **Interactive Markdown Reader:** Structured rendering for `#`/`##`/`###` headings, monospace code blocks, and indented lists.
- **Clickable Wiki-Links & Backlinks:** Clicking any `[[target]]` link or backlink pill instantly navigates to that page.
- **Page Editor:** Toggle `✏️ Edit Page` to modify titles and markdown content with `💾 Save`.
- **🌱 Seed Demo:** Bootstrap sample architecture docs, wiki links, and ADRs with a single click.
- **📂 Ingest Docs:** 1-click batch ingestion of `mcp-central-docs` sections (`Standards`, `Patterns`, `Operations`, `Troubleshooting`, `Architecture`) or any custom markdown directory.
- **🏛️ ADR Matrix:** Filterable table tracking architectural decisions across `proposed`, `accepted`, `rejected`, and `superseded` states.
- **🕸️ Wiki Graph:** Interactive topology view mapping outgoing links and incoming backlinks.

---

## CLI & MCP Server (`wikicraft`)

```bash
# General status and counts
wikicraft status

# Markdown Pages
wikicraft page-save <id> <title> <content>
wikicraft page-get <id>
wikicraft page-list
wikicraft page-delete <id>
wikicraft search <query>
wikicraft backlinks <id>

# Architecture Decision Records (ADRs)
wikicraft adr-save <id> <title> [status] [context] [decision] [consequences] [--superseded-by <id>]
wikicraft adr-get <id>
wikicraft adr-list

# Directory Import & Ingestion
wikicraft tree-import <directory-path>

# Bootstrap Demo Data
wikicraft seed-demo

# Event Bus Inbox
wikicraft inbox-drain

# Run as Stdio MCP Server for AI Coding Agents
wikicraft mcp

# Inspect Tool Specifications
wikicraft tools [--markdown]
```
