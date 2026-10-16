---
flow: build
priority: 3
deps: []
---

# 2147: Report every section batch article is asked for

Status: OPEN.

Milestone: 0.9.3

## Outcome

`batch article --sections` returns each requested section or an outcome saying why it is missing, adds no section nobody asked for, and refuses a section it never supports.

## Evidence

Reported 2026-10-16 by the research team on the released 0.9.2.

- `biomcp --json batch article 9021520,9096355 --mode detail --sections fulltext,assets` returns full text but no assets and no outcome for that section, and includes an unrequested `tldr` section.
- The `get article` help lists annotations, indexing, fulltext, tldr, assets, `asset <asset-key>` and all (`src/cli/article/mod.rs:310`). Batch builds items in `src/entities/article/batch.rs` and parses sections in `src/cli/system/batch.rs`.

## Work

- Reproduce on main first.
- Support `assets` in batch, or refuse it at parse time naming the supported sections. Pick one and record why in this ticket.
- Return only requested sections, and give each requested section that comes back empty an outcome.
- State the supported sections in the batch help and the CLI reference.

## Acceptance

1. A test of `--sections fulltext,assets` returns both sections with outcomes, or refuses before any request; it fails on current main.
2. A test shows no `tldr` section unless asked for.
3. The changelog carries a bullet for this ticket.
