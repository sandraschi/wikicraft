# WikiCraft Quick Reference

## CLI Commands

```bash
# General status
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

# Directory Import
wikicraft tree-import <directory-path>

# Event Bus
wikicraft inbox-drain

# MCP Server
wikicraft mcp
```
