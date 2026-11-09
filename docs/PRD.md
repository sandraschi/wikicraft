# WikiCraft: Product Requirements Document

## 1. Executive Summary

`wikicraft` is the central documentation, knowledge repository, and Architecture Decision Record (ADR) management system of the Back-Office Craft Suite.

## 2. Key Capabilities

- **Local Markdown Tree Ingestion:** Automatically crawls local repositories and projects, extracting wiki links (`[[link]]`) and indexing documentation into SQLite FTS5.
- **Fast Full-Text Search (FTS5):** Sub-millisecond snippet queries without cloud dependencies or telemetry leaks.
- **ADR Governance:** Formally manages architectural decisions with status transitions (`proposed`, `accepted`, `rejected`, `superseded`) and broadcast notifications over the `craft-biz` bus.
- **Zero-Cloud Air-Gapped Operation:** Eliminates external vendor subscription fees and preserves confidential system design data.

## 3. Replaces & Cost Savings

- **Replaces:** Confluence (\$60/seat/mo), Notion Team (\$100/seat/mo), Slite (\$10/seat/mo).
- **Cost Reduction:** 100% elimination of cloud subscription fees, zero latency, guaranteed uptime during network partitioning.
