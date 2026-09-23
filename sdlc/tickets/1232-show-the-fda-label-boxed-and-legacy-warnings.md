# Show the FDA label boxed and legacy warnings

Filed from `sdlc/issues/2026-09-23-fda-label-boxed-and-legacy-warnings-are-never-shown.md`. Blocks 0.9.1.

## Problem

`src/entities/drug/label.rs:362` and `:396` read only
`warnings_and_cautions`. No code in `src/` or `templates/` reads
`boxed_warning`, and older-format labels keep their warnings in `warnings`.
A boxed warning never reaches the reader for any drug, and
`biomcp get drug vincristine safety --region us` prints "FDA label
warnings: No data found" for a label that was fetched successfully.

## Design

1. Read `boxed_warning`, then `warnings_and_cautions`, then `warnings`, in
   that order, in both label-reading sites.
2. Render the boxed warning in its own leading block, visually ahead of the
   other warnings, in the Markdown and JSON outputs. Update the label
   templates if they enumerate sections.
3. Add fixtures: a current-format label carrying a boxed warning, and an
   older-format label whose warnings live only in `warnings`. Tests assert
   the boxed warning leads and the legacy field is read.

## Acceptance

- Fixture tests prove all three fields reach the output on yellow, offline.
- Full yellow gate at the head SHA: `make lint`, `make test`, `make spec`.
- A record lands in `sdlc/records/`; the issue file gains a Resolved section.

## Review

- Design review: pending
- Code review: pending
