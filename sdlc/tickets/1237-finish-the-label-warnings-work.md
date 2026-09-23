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
3. **One heading set**: "Boxed Warning" and "FDA boxed warning" become one
   heading; legacy text renders under a neutral "Warnings" heading; when
   both sections are requested the boxed warning prints once.
4. **Tests and docs**: a fixture with both `warnings_and_cautions` and
   `warnings` pins the precedence; a JSON test asserts `boxed_warning`;
   multi-string and truncated boxed warnings are tested;
   `docs/user-guide/drug.md` and `docs/sources/openfda.md` mention boxed
   warnings. The two render-order tests move out of
   `src/render/markdown/drug/tests.rs` into a new test file and the size
   inventory entry for it is dropped; the `provenance.rs` reason text is
   corrected to describe `get.rs:1062-1070`.

## Acceptance

- All listed tests pass on yellow; the inventory entry is gone.
- Docs updated; full yellow gate at the head SHA; the issue file gains a
  Resolved section; a record lands.

## Review

- Design review: pending
- Code review: pending
