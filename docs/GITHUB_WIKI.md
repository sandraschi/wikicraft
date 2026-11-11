# GitHub Wiki Integration Guide

WikiCraft includes native bidirectional synchronization with GitHub Wiki repositories (`<repo>.wiki.git`). This document explains the architecture, workflows, and conventions.

---

## 1. How GitHub Wikis Work Under the Hood

Every GitHub repository with wikis enabled has a dedicated Git repository:
```
https://github.com/<owner>/<repo>.wiki.git
```

### Key GitHub Wiki Properties
- **Git Backend:** It is a standard Git repository containing flat Markdown files.
- **Link Syntax:** GitHub Wiki natively parses `[[Page Name]]` and `[[Target|Custom Label]]` wiki-links — identical to WikiCraft's parser.
- **`Home.md`:** The landing page rendered when visiting `https://github.com/<owner>/<repo>/wiki`.
- **`_Sidebar.md`:** A special file GitHub renders as the navigation sidebar.
- **`_Footer.md`:** A special file rendered at the bottom of every wiki page.

> [!IMPORTANT]
> **First-Time Initialization:** GitHub only instantiates the underlying `.wiki.git` repository when the first page (`Home`) is created via the web interface at `https://github.com/<owner>/<repo>/wiki`. Once initialized, WikiCraft can clone, pull, and push to it freely.

---

## 2. Default Repository Resolution

For `sandraschi` fleet development:
- Passing a plain repository name (e.g. `wikicraft`, `taskcraft`, `codercraft`) automatically resolves to `sandraschi/<repo>`.
- Any external or organization repository can be targeted explicitly (e.g. `owner/repo`).

---

## 3. Workflows

### A. Pull & Ingest (`gh-pull`)

Pulls remote wiki changes and ingests all markdown files into WikiCraft's SQLite WAL store and FTS5 search index.

**Desktop UI:**
1. Click **`🌐 GH Wiki ▼`** in the top bar.
2. Enter repository name (e.g. `wikicraft` or `sandraschi/wikicraft`).
3. Click **`⬇️ Pull Wiki`**.

**CLI / MCP:**
```bash
# Via CLI
wikicraft-mcp gh-pull wikicraft

# Via MCP tool
# Tool: wiki_gh_pull
# Arguments: { "repo": "sandraschi/wikicraft" }
```

**What Happens:**
1. WikiCraft checks `%LOCALAPPDATA%\wikicraft\wikis\<owner>_<repo>\`.
2. If absent, clones `https://github.com/<owner>/<repo>.wiki.git`.
3. If present, runs `git pull --rebase`.
4. Ingests all `.md` files into SQLite, indexing text in FTS5 and extracting `[[wiki-links]]`.

---

### B. Export & Push (`gh-push`)

Exports all pages and ADRs from WikiCraft into the GitHub Wiki repository, automatically generating navigation, committing, and pushing via Git.

**Desktop UI:**
1. Click **`🌐 GH Wiki ▼`**.
2. Click **`⬆️ Push Wiki`**.

**CLI / MCP:**
```bash
# Via CLI
wikicraft-mcp gh-push wikicraft --message "docs: sync release architecture"

# Via MCP tool
# Tool: wiki_gh_push
# Arguments: { "repo": "sandraschi/wikicraft", "message": "docs: sync" }
```

**What Happens:**
1. Exports all stored pages to `<safe-filename>.md` in the wiki directory.
2. Automatically generates **`_Sidebar.md`**:
   - `### Documentation`: links to all articles.
   - `### Architecture Decisions`: links to all ADRs with their statuses (`accepted`, `proposed`, etc.).
3. Exports ADRs as individual documents.
4. Automatically generates **`_Footer.md`** with WikiCraft attribution.
5. Stages all changes (`git add -A`), commits with the provided message, and pushes to `origin/master` (or `main`).

---

### C. Status Inspection (`gh-status`)

Checks whether the local wiki repository is cloned and initialized, and inspects the latest Git commit.

**Desktop UI:**
Click **`ℹ️ Status`** inside the `🌐 GH Wiki` drawer.

**CLI / MCP:**
```bash
wikicraft-mcp gh-status wikicraft
```

Output:
```json
{
  "initialized": true,
  "last_commit": "0fad0c2 docs: update wiki architecture",
  "local_dir": "C:\\Users\\sandr\\AppData\\Local\\wikicraft\\wikis\\sandraschi_wikicraft",
  "remote_url": "https://github.com/sandraschi/wikicraft.wiki.git",
  "repo": "sandraschi/wikicraft"
}
```
