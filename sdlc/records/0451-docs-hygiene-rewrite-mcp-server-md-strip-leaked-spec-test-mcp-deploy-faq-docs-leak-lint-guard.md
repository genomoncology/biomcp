---
base: 2fe3fd2eeb885fc73f75d20c2f6050434aba63ed
head: 16a4c041169efbbbae10f0cee3072c665424a384
---
`docs/reference/mcp-server.md` is published on biomcp.org but is a mustmatch/spec-style test masquerading as documentation: 9 Python `assert` blocks (~74 asserts) that `read_text()` Rust source / `build.rs` / `tests/*` and assert literal strings exist. It is nonsense to a reader, and it's the page that should answer "how do I run BioMCP as an MCP server?" — a question a St. Jude developer just asked by email. A full docs audit confirms this is the **only** page with test leakage (the Python blocks are NOT executed by mkdocs — `plugins:` is `search` only — so they ship verbatim as dead text). Replace it with real deployment docs, add an FAQ entry for the recurring deploy/auth question, fix a few low-severity hygiene items found in the same audit, and add a guard so executable test code cannot leak into `docs/` again.

Imported from March ticket 451. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/451-docs-hygiene-rewrite-mcp-server-md-strip-leaked-spec-test-mcp-deploy-faq-docs-leak-lint-guard
