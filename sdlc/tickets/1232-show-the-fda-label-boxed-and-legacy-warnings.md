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

1. `boxed_warning` is its own field, rendered always when present. The
   fallback chain `warnings_and_cautions` then `warnings` applies only to
   the non-boxed warnings text, in both label-reading sites
   (`src/entities/drug/label.rs:362` raw mode and `:396` safety path).
   A modern label carrying both fields renders both.
2. Render the boxed warning in its own leading block, visually ahead of the
   other warnings, in the Markdown and JSON outputs: ahead of
   "### Warnings and Precautions" (`templates/drug.md.j2:22-26`) and ahead
   of "### FDA label warnings" (`drug_regulatory.rs:342`). Populate the
   boxed field in summary mode as well, not only raw/safety mode.
3. Extend the section-outcome contributor check at `get.rs:1051-1058`, which
   keys only on `us_safety_warnings` non-empty, to include the boxed field;
   otherwise a boxed-only label renders the block yet still reports the
   safety section as empty-sourced.
4. `extract_inline_label`'s emptiness early-return (`label.rs:375-381`)
   must include the boxed field, or boxed-only labels still yield `None`.
5. Fixtures follow the inline `serde_json::json!` convention
   (`label/tests/extraction.rs:58-74`) with render pins at
   `render/markdown/drug/tests.rs:344,413`: a current-format label with a
   boxed warning, an older-format label with only `warnings`, and the
   common middle case with both `boxed_warning` and
   `warnings_and_cautions`. Preserve the pinned unit at
   `render/markdown/drug/tests.rs:413`.

`spec/entity/drug.md` pins no warnings wording, so no spec page changes.

## Acceptance

- Fixture tests prove all three fields reach the output on yellow, offline.
- Full yellow gate at the head SHA: `make lint`, `make test`, `make spec`.
- A record lands in `sdlc/records/`; the issue file gains a Resolved section.

## Review

- Design review: ACCEPT 2026-09-23 (gpt-5.6-sol, medium) — both P1
  clarifications folded in before implementation
- Code review: ACCEPT 2026-09-23 (gpt-5.6-sol, medium) — P2 truncate fix
  applied in 934f44aa and 88cdca4b
- Verification: yellow gate at abade1b4 lint/spec OK and `make test` green
  (one documented GenCC load flake on the first run, 3/3 isolated passes);
  see `sdlc/records/1232-show-the-fda-label-boxed-and-legacy-warnings.md`
