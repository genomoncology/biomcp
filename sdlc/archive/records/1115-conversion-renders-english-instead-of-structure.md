---
flow: build
priority: 4
---

# Preserve the complete provider trial summary outside Markdown rendering

## Outcome

`Trial.summary` and trial JSON retain the complete CTGov or NCI brief summary,
after only the existing outer-whitespace/blank-value normalization. Markdown
continues to show the same bounded summary as it does before this ticket through
one renderer-owned, directly testable summary function that ticket 1113 can
later correct.

## Current facts and reproducer

`src/transform/trial.rs` owns `first_n_sentences` and `truncate_summary`.
`from_ctgov_study` and `from_nci_trial` both call `truncate_summary` while
constructing `Trial.summary`; the `Trial` model then serializes that shortened
string directly. `src/render/markdown/trial.rs` passes `trial.summary` unchanged
to `templates/trial.md.j2`. Conversion has therefore destroyed provider data
before either JSON or Markdown chooses a presentation policy.

The current focused test makes the loss an expected conversion behavior:

```text
source:    Sentence one. Sentence two. Sentence three.
converted: Sentence one. Sentence two.
```

The byte branch likewise keeps at most 500 UTF-8 content bytes and appends
`...`. `cargo test --no-default-features transform::trial::tests::` passed all
21 selected tests on 2026-09-04, including that assertion and the NCI conversion
assertion that only checks for the retained prefix. The behavior dates to
commit `09b789d22`.

This is not only a synthetic possibility. Recorded provider data includes
brief summaries longer than the display cap: the largest observed CTGov value
in the current corpus is 1,739 characters
(`testdata/sources/ctgov/search_phelan_limit5_20260811.json`), and the largest
observed NCI value is 1,463 characters
(`testdata/sources/nci_cts/search_melanoma.json`). A conversion or JSON caller
cannot recover their omitted text today.

The original ticket also claimed every converted age had to be parsed back out
of `age_range`. That is no longer true for CTGov: since commit `49cd4485`,
conversion also creates `TrialEligibility { minimum_age, maximum_age, ... }`.
The remaining age design is not a small part of this change. Default trial get
deliberately removes section-scoped `eligibility`, NCI still puts its now-
correct nested bounds only into prose `age_range`, and ticket 1116 already owns
the cross-provider parsed-plus-source-text age model used by filtering and
output. Age fields, filtering, and age Markdown are therefore excluded here;
this ticket no longer asserts that both providers have the same age defect.

## Test-first design

Add the failing data-preservation assertions before relocating the policy.

1. Add paired CTGov and NCI conversion tests using the existing recorded
   fixtures named above. Read the expected `briefSummary` / `brief_summary`
   directly from each provider object, assert that it genuinely crosses the
   current sentence or byte boundary, then require `Trial.summary` and
   `serde_json::to_value(&trial)["summary"]` to equal that complete normalized
   value. Add a small synthetic blank/outer-whitespace control so this ticket
   does not turn blank input into `Some("")` or promise byte-for-byte
   preservation of insignificant leading/trailing whitespace.
2. Before changing conversion, add renderer regressions with focused synthetic
   values. Through `trial_markdown`, require the Summary section to contain
   exactly the current two-sentence/500-byte projection and not the omitted
   tail. Move the existing ordinary three-sentence and multibyte cases into the
   renderer tests as the byte-for-byte baseline; this ticket must preserve the
   current absence of a marker when sentence counting alone omits text.
3. Move the sentence and summary byte policy into one summary-specific helper
   beside `trial_markdown` (for example,
   `bounded_trial_summary(&str) -> String`). Compute the optional rendered
   summary once in `trial_markdown` and pass that value to the template. Tests
   in the renderer's child test module must be able to exercise the helper
   directly. Do not implement the same policy as a template filter or in each
   provider converter.
4. Make both converters retain the complete normalized source value. Remove
   the summary-specific sentence/bounding helpers from conversion; a generic
   UTF-8 helper may remain there while `format_conditions` still owns a
   separate conversion-time condition-list bound.

The seam supplied to ticket 1113 is exact: there is one renderer-local bounded-
trial-summary function used by `trial_markdown`; `Trial.summary` and its JSON
serialization remain full. Ticket 1113 changes that function's boundary and
marker behavior, not provider conversion, `Trial`, or the template.

## Scope, compatibility, and trust boundaries

The `summary` JSON field keeps its name, optionality, and string type but
intentionally becomes longer whenever conversion previously omitted source
text. Consumers that assumed a 503-byte maximum must instead apply their own
presentation bound. Markdown is compatibility-sensitive in this ticket and
must remain byte-identical for the focused ordinary, sentence-limited, byte-
limited, blank, and multibyte cases. Ticket 1113's later abbreviation/marker
changes are not part of this baseline.

Provider summary text is untrusted data. This ticket does not interpret,
rewrite, or execute it and does not broaden Markdown exposure: Markdown still
receives only the current bounded projection, while JSON string serialization
performs its existing escaping. Full JSON size remains bounded indirectly by
the shared 8 MiB provider-response limit, but there is no honest per-field cap.
Do not introduce a silent replacement cap under the word "complete"; any future
resource limit would need an explicit truncation/completeness contract.

Do not change age fields, age filtering, the condition-list cap, the template's
section layout, recorded captures, provider requests, or documentation. Current
docs promise a summary card but do not specify its sentence algorithm or byte
count, so no mustmatch or user-guide update is required for this relocation.

## Dependencies

No implementation prerequisite. Ticket 1113 depends on this ticket and must
land afterward because it changes the renderer seam established here. Ticket
1116 owns the separate age representation/filtering outcome; neither ticket is
part of 1115's implementation.

## Acceptance

- Failing-first recorded CTGov and NCI conversion tests prove the complete
  normalized provider summary reaches both `Trial.summary` and JSON.
- Renderer/helper tests prove Markdown still applies the existing two-sentence
  and 500-content-byte policy, remains UTF-8 safe, and preserves the pre-1113
  bytes for sentence-only and byte-only truncation.
- Exactly one renderer-local bounded-summary seam feeds the trial template, and
  neither provider conversion shortens `Trial.summary`.
- No age, provider request, capture, spec, user-guide, or unrelated output
  change is included.
- Focused trial transform and Markdown tests pass, then the repository gates
  pass exactly:

  ```text
  make lint
  make test
  make spec
  ```

## Review

- Design review: ACCEPT (2026-09-04). The data-loss claim is supported by both
  conversion paths and by the current 21-test transform lane: sentence and
  UTF-8 byte shortening happen before `Trial.summary` is serialized, while the
  shared Markdown renderer currently forwards that shortened value unchanged.
  The recorded-corpus maxima are reproducible at 1,739 CTGov characters and
  1,463 NCI characters, and both named search fixtures can drive direct
  provider-value, converted-value, and serialized-value comparisons without a
  new capture. Keeping only existing trim/blank normalization gives JSON a
  clear compatible type/optionality contract while honestly changing its
  former incidental 503-byte size ceiling; serde escaping and the shared 8 MiB
  response-wide limit preserve the stated trust boundary without inventing a
  per-field completeness cap. Moving the exact old two-sentence/500-byte policy
  to one directly tested helper beside `trial_markdown` preserves current
  Markdown bytes and gives accepted ticket 1113 precisely one downstream seam
  for its intentional abbreviation/marker changes. The ordinary,
  sentence-only, byte-only, blank/outer-whitespace, and multibyte controls are
  feasible in the existing transform and renderer child-test modules. Removing
  stale age work is correct: CTGov already has structured eligibility that is
  section-scoped, NCI does not share that representation, and ticket 1116 owns
  the cross-provider age outcome. There is no implementation prerequisite;
  1113 is the exact dependent, 1116 is independent, and rollback is confined to
  the two converter assignments plus the renderer-local helper/wiring and their
  focused tests, with no model, request, fixture, template-layout, spec, or docs
  migration required.
- Implementation evidence (2026-09-04): failing-first coverage initially did
  not compile because the required renderer seam was absent. The implementation
  now retains the complete normalized summaries from recorded CTGov
  `NCT07119606` and NCI `NCT05929768` records in both `Trial.summary` and JSON,
  while one renderer-local `bounded_trial_summary` function preserves the
  existing ordinary sentence and UTF-8 byte-cap Markdown bytes. The focused
  renderer lane passed 10 tests, the focused trial-transform lane passed 23
  tests, formatting and `cargo clippy --locked --no-default-features --lib --
  -D warnings` passed. The single `spec/entity/trial.md` page could not be run
  validly outside its routine fixture lifecycle: both direct offline attempts
  stopped on unavailable CTGov responses before reaching a summary assertion.
  A subsequent primary `make lint` run rejected four invented provider-shaped
  JSON values in the normalization test under the source-capture contract. The
  remediation removed those values and tests trimming, blank rejection, and
  absence through a pure `normalize_summary` seam shared by both converters;
  the recorded CTGov and NCI preservation tests remain unchanged. The direct
  source-capture checker, all 23 focused trial-transform tests, formatting, and
  diff checks then passed. No full repository gate was run during
  implementation.
- Code review: ACCEPT (2026-09-04). The reviewer verified complete normalized
  CTGov and NCI summaries in `Trial`/JSON, the single renderer-local shortening
  seam, byte-identical pre-1113 Markdown behavior, UTF-8 safety, unchanged
  null/blank handling, and the absence of age/schema/provider-request changes.
  After the source-capture rejection, independent re-review accepted the pure
  shared normalization seam and confirmed that all invented provider-shaped
  JSON was removed while recorded provider tests remained intact. No blocking
  findings remain.

## Completed 2026-09-04

Both trial converters now retain the complete normalized provider summary in
`Trial.summary` and JSON. Markdown alone applies the legacy two-period and
500-byte projection through one renderer-owned `bounded_trial_summary` seam,
which is the accepted handoff for dependent ticket 1113.

Final primary gates passed on the independently accepted tree: `make lint`,
including source-capture governance and quality ratchets; `make test`, including
the complete offline Rust lane, 883 Python tests passed and 3 skipped, and
strict documentation; and `make spec`, including the routine CTGov-managed
trial contracts, 38 isolation contracts, fixture cleanup, and the 8-case static
lane.
