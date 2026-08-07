---
flow: devops
priority: 10
---
# Fix uv project-build hang in make check so local release-gate completes

Both the canonical `make check` run and a targeted spec run timed out during ticket 327's code review. The Rust lint and release build completed, then `uv sync --extra dev` / `Building biomcp-cli @ file://...` hung past 1200s and 600s respectively, never reaching pytest, mkdocs, or the quality-ratchet lanes. Pure-Python ratchets via `uv run --no-project ...` ran in seconds.

Completed under March on 2026-04-27, as March ticket 330. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/330-fix-uv-project-build-hang-in-make-check-so-local-release-gate-completes
