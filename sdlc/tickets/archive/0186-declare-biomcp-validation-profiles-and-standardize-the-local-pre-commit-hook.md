---
flow: build
priority: 10
---
# Declare biomcp validation profiles and standardize the local pre-commit hook

Biomcp already has tiered validation lanes — `make check`, `make spec-pr`, full `make spec`, `make test-contracts` — but the build-flow prompts point at hardcoded `make check` / `make spec` commands and re-run them at every step transition. The path to stop that duplication is a small config file that declares which biomcp lane corresponds to each build-flow profile tier, so that once the march-side profile system is live the build flow consumes the right tier per step instead of the heaviest lane six times per ticket.

Completed under March on 2026-04-13, as March ticket 186. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/186-declare-biomcp-validation-profiles-and-standardize-the-local-pre-commit-hook
