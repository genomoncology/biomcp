---
base: 200372481b09d829fcb09ee5d43bb39b408a9d9d
head: d55e753802c745279be128a8976eedb2292d7bb2
---
The docs site documents the two newest install paths — Homebrew (`docs/getting-started/installation.md` → `## Homebrew`) and Docker/GHCR (`installation.md` → `## Option 4: Docker image`, plus `docs/reference/mcp-server.md` → Docker stdio section) — and both pages are in the mkdocs nav. The **README**, which is the GitHub landing page and the first install reference most users see, was never updated: its `## Installation` section lists nine methods (Binary, PyPI, plugin, Codex, Claude Desktop, skills, MCP clients, Remote HTTP, From source) but **neither Homebrew nor Docker**. This ticket brings the README to parity with the docs site so the two shipped deployment methods are discoverable from the repo front page.

Imported from March ticket 480. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/480-readme-document-homebrew-and-docker-install-paths
