---
base: 9ea4f6a855d15edebc09c28f226a8e23495ecd58
head: 5ac5dec78b1623ee28489bef2d58d6b94bcdfe7a
---
Docs-only hotfixes on the publish flow currently pay a full `make check` tax at runtime verify+merge — 1,481 Rust unit tests + lint + quality-ratchet compiled from a cold worktree cache — even when the ticket only edits a single markdown file under `docs/`. Evidence: ticket 201 (one-line scrub of `pip install biomcp-python` to the canonical curl installer in `docs/blog/daraxonrasib-six-commands.md`) spent 17-23 minutes in `verify+merge`, almost all of it running cargo tests that had nothing to do with the change.

Imported from March ticket 205. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/205-add-docs-only-validation-profile-and-wire-it-to-publish-flow
