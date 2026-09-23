# Finish the label warnings work

Filed from `sdlc/issues/2026-09-23-fda-label-warning-follow-ups.md`. Before 0.9.1.

## Design

1. **Cap the safety-path warnings**: `extract_label_warnings_text`
   (`label.rs:396-403`) applies the same 2,000-character cap as the boxed
   warning beside it. Test included.
2. **A way to the full text**: the truncation note carries the DailyMed
   link built from `label_set_id`, not only "(truncated, N chars total)";
   the link no longer depends on `drug.label` being present
   (`render/markdown/evidence.rs:352`).
3. **One heading, deduped in the renderer**: "Boxed Warning" and
   "FDA boxed warning" become one heading; legacy text renders under a
   neutral "Warnings" heading (extraction already merges modern and
   legacy fields into one value at `label.rs:364-366,401-402`). Both data
   fields stay independent for the separate sections and JSON; the
   Markdown renderer suppresses the safety copy of the boxed warning when
   the label section already rendered it — deduplication happens in
   rendering (`src/render/markdown/drug.rs:19-35,99-100`), not section
   parsing.
4. **Tests and docs**: a fixture with both `warnings_and_cautions` and
   `warnings` pins the precedence; a JSON test asserts `boxed_warning`;
   multi-string and truncated boxed warnings are tested. Two named
   regression tests: one boxed heading when `label` and `safety` are
   combined, and a truncated safety warning containing the exact set-id
   DailyMed URL when `drug.label` is absent.
   `docs/user-guide/drug.md` and `docs/sources/openfda.md` mention boxed
   warnings. The two render-order tests move out of
   `src/render/markdown/drug/tests.rs` into a new test file (about 141
   lines, returning it under its 943-line floor) and the size inventory
   entry for it is dropped; the retained provenance inventory reason at
   `tools/rust-source-size-inventory.json:319-326` is corrected to say
   three provenance test fixtures initialize the new field, since the
   contributor logic it describes lives at `get.rs:1062-1070`.

## Acceptance

- All listed tests pass on yellow; the inventory entry is gone.
- Docs updated; full yellow gate at the head SHA; the issue file gains a
  Resolved section; a record lands.

## Review

- Design review: REJECT once 2026-09-23 (gpt-5.6-sol, medium) — three
  findings folded in: deduplication belongs in the Markdown renderer
  with both data fields preserved, the two named regression tests are
  added (combined-section single heading; truncated safety warning with
  the exact DailyMed URL and no `drug.label`), and the retained
  inventory reason is corrected. Second review pending.
- Code review: pending
