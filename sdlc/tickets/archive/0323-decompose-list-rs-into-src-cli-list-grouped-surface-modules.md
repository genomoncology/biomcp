---
flow: build
priority: 5
---
# Decompose list.rs into src/cli/list/ grouped surface modules

`src/cli/list.rs` is 1,534 lines and currently mixes the top-level router with 23 hard-coded page builders and an inline test block. The command works, but its static reference pages have no ownership boundaries, so even a one-page edit requires navigating a giant flat file and risks re-growing the CLI reference surface past the architecture cap.

Completed under March on 2026-04-27, as March ticket 323. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/323-decompose-list-rs-into-src-cli-list-grouped-surface-modules
