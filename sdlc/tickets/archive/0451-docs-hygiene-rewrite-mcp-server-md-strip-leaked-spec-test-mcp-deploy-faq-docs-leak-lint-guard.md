---
flow: quickfix
priority: 5
---
# Docs hygiene: rewrite mcp-server.md (strip leaked spec test) + MCP-deploy FAQ + docs-leak lint guard

`docs/reference/mcp-server.md` is published on biomcp.org but is a mustmatch/spec-style test masquerading as documentation: 9 Python `assert` blocks (~74 asserts) that `read_text()` Rust source / `build.rs` / `tests/*` and assert literal strings exist. It is nonsense to a reader, and it's the page that should answer "how do I run BioMCP as an MCP server?" — a question a St. Jude developer just asked by email. A full docs audit confirms this is the **only** page with test leakage (the Python blocks are NOT executed by mkdocs — `plugins:` is `search` only — so they ship verbatim as dead text). Replace it with real deployment docs, add an FAQ entry for the recurring deploy/auth question, fix a few low-severity hygiene items found in the same audit, and add a guard so executable test code cannot leak into `docs/` again.

Completed under March on 2026-06-25, as March ticket 451. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/451-docs-hygiene-rewrite-mcp-server-md-strip-leaked-spec-test-mcp-deploy-faq-docs-leak-lint-guard
