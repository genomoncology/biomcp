---
flow: quickfix
priority: 5
---
# README: document Homebrew and Docker install paths

The docs site documents the two newest install paths — Homebrew (`docs/getting-started/installation.md` → `## Homebrew`) and Docker/GHCR (`installation.md` → `## Option 4: Docker image`, plus `docs/reference/mcp-server.md` → Docker stdio section) — and both pages are in the mkdocs nav. The **README**, which is the GitHub landing page and the first install reference most users see, was never updated: its `## Installation` section lists nine methods (Binary, PyPI, plugin, Codex, Claude Desktop, skills, MCP clients, Remote HTTP, From source) but **neither Homebrew nor Docker**. This ticket brings the README to parity with the docs site so the two shipped deployment methods are discoverable from the repo front page.

Completed under March on 2026-07-08, as March ticket 480. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/480-readme-document-homebrew-and-docker-install-paths
