---
base: 7865dc85cb3c6ab08072bebee3472cb7f0332286
head: 7dd551b32b0f87094676a40fd7df235a8736fb30
---
Both the canonical `make check` run and a targeted spec run timed out during ticket 327's code review. The Rust lint and release build completed, then `uv sync --extra dev` / `Building biomcp-cli @ file://...` hung past 1200s and 600s respectively, never reaching pytest, mkdocs, or the quality-ratchet lanes. Pure-Python ratchets via `uv run --no-project ...` ran in seconds.

Imported from March ticket 330. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/330-fix-uv-project-build-hang-in-make-check-so-local-release-gate-completes
