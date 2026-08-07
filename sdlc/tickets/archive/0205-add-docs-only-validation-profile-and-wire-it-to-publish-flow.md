---
flow: quickfix
priority: 6
---
# Add docs-only validation profile to biomcp

Docs-only hotfixes on the publish flow currently pay a full `make check` tax at runtime verify+merge — 1,481 Rust unit tests + lint + quality-ratchet compiled from a cold worktree cache — even when the ticket only edits a single markdown file under `docs/`. Evidence: ticket 201 (one-line scrub of `pip install biomcp-python` to the canonical curl installer in `docs/blog/daraxonrasib-six-commands.md`) spent 17-23 minutes in `verify+merge`, almost all of it running cargo tests that had nothing to do with the change.

Completed under March on 2026-04-15, as March ticket 205. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/205-add-docs-only-validation-profile-and-wire-it-to-publish-flow
