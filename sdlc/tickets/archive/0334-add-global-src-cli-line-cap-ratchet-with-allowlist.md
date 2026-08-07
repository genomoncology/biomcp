---
flow: quickfix
priority: 6
---
# Add global src/cli line-cap ratchet with allowlist

The 700-line cap on `src/cli/**/*.rs` is a durable architecture rule in `architecture/technical/cli-module-decomposition.md`, but only the recently decomposed areas have structure ratchets (search_all, health, suggest, skill, list, article tests, benchmark). `make check` can pass while new or out-of-scope files exceed the cap. The 327 review found six current over-cap files that no ratchet covers:

Completed under March on 2026-04-28, as March ticket 334. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/334-add-global-src-cli-line-cap-ratchet-with-allowlist
