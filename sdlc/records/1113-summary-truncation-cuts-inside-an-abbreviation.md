---
flow: build
priority: 5
deps: ["1115"]
---

# Mid-sentence clinical abbreviations consume the summary's sentence budget

## Outcome

The bounded Markdown trial summary keeps the clause after each supported
mid-sentence abbreviation, keeps its existing two-sentence and 500-content-byte
limits, and ends in `...` whenever either limit omitted source text.

## Current facts and reproducer

`src/transform/trial.rs` currently implements `first_n_sentences` by counting
every `.` followed by ASCII whitespace or end-of-input. `truncate_summary`
first takes two such "sentences" and then keeps at most 500 bytes plus the
existing `...` suffix. Both `from_ctgov_study` and `from_nci_trial` call that
function, so the already-shortened value reaches JSON and Markdown alike.
`templates/trial.md.j2` renders the value without any further summary policy.

The original ticket's one-abbreviation reproducer was inaccurate: one `pts.`
only consumes one of the two boundary counts and does not itself stop the
summary. A real sentence followed by a mid-sentence abbreviation does fail.
For example, the current helper returns:

```text
input:  Background is established. This study enrolls 40 pts. with relapsed disease and compares two regimens. The endpoint is survival.
output: Background is established. This study enrolls 40 pts.
```

The same construction stops at each of `vs.`, `approx.`, `e.g.`, `i.v.`, and
`Dr.`. It discards the rest of the second real sentence and the output has no
truncation marker. The existing unit test proves the related marker gap too:
`"Sentence one. Sentence two. Sentence three."` is expected to become
`"Sentence one. Sentence two."`, even though source text was omitted.

This behavior and its test date to commit `09b789d22`; the current focused test
still passes unchanged. The recorded trial corpus contains at least one
mid-sentence `e.g.` summary, but that summary has only one real sentence and
does not reproduce the two-boundary failure. Synthetic unit inputs are the
smallest deterministic evidence; this ticket does not need a new provider
capture.

## Test-first design

After dependency 1115 has moved summary shortening to the Markdown renderer,
add the renderer-owned regressions first and show that they fail before changing
the boundary logic.

1. Table-drive the six supported forms (`pts.`, `vs.`, `approx.`, `e.g.`,
   `i.v.`, and `Dr.`) with one real sentence before the abbreviation, a complete
   second sentence containing it, and a third sentence. For every row, require
   the complete second sentence, omission of the third, and one terminal `...`
   marker.
2. Exercise the public Markdown path with a `Trial` carrying one of those full
   summaries. Require the rendered Summary section to retain the post-
   abbreviation disease/design clause. The converted/JSON value remains the
   full provider summary established by 1115.
3. Pin the boundary controls: exactly two ordinary sentences are unchanged and
   unmarked; three ordinary sentences keep two and are marked; an ordinary word
   ending in the same letters (for example `attempts.`) remains a sentence
   boundary; and a supported abbreviation that genuinely terminates a sentence
   (for example `Route was i.v. Participants recovered. Follow-up continued.`)
   still counts as the first boundary, keeps `Participants recovered.`, and
   omits `Follow-up continued.`.
4. Retain the current multibyte byte-cap case and strengthen it to require valid
   UTF-8, at most 500 content bytes plus the three-byte ASCII marker, and exactly
   one terminal marker when sentence and byte limits both apply.

Implement abbreviation recognition as boundary policy, not as replacement or
cleanup of source text. Match complete abbreviation tokens ASCII-case-
insensitively rather than suffixes of longer words. For this bounded set, a
matched token continues the sentence when the next lexical character is
lowercase or numeric; `Dr.` also continues before an uppercase name. At end of
input, or before an uppercase character for the other five forms, its final
period can still terminate the sentence. An unconditional
`ends_with("pts.")` allowlist would break the control cases. No new parsing
dependency is justified for these six forms.

## Scope and compatibility

This ticket changes only bounded Markdown trial-summary presentation after
1115. It does not truncate the converted value or JSON, change provider
parsing, add a summary field, or edit recorded captures. CTGov and NCI share the
same `Trial` renderer, so provider-specific copies of the boundary code are out
of scope.

Keep the two-sentence limit, the 500-byte content limit, the existing ASCII
`...` marker, whitespace trimming, and UTF-8 safety. When a retained sentence
ends in `.`, replace that terminator with `...` rather than producing four
periods. Do not broaden this ticket into general sentence segmentation:
question marks, exclamation marks, quoted terminators, initials,
decimal/version text, and abbreviations outside the six listed forms retain
their existing behavior unless a control test is needed to prevent regression.

No documentation currently specifies the sentence algorithm or byte count;
the source page only promises a trial summary card, so no user documentation or
mustmatch fixture change is required.

## Dependencies

Dependency: 1115. It moves the full source summary out of conversion and makes
shortening renderer-owned while intentionally preserving current rendered
bytes. Specifically, 1115 must leave the unabridged provider text in
`Trial.summary` and JSON, and route the Markdown template's summary value
through one renderer-local bounded-summary function or equivalent testable
seam. Landing 1113 afterward fixes that seam once and avoids changing a helper
that 1115 is already required to relocate. Ticket 1115's byte-identical baseline
is the pre-1113 output; 1113 then intentionally changes only the abbreviation
and truncation-marker cases described here.

No later trial-field ticket is a prerequisite.

## Acceptance

- All six table cases retain their post-abbreviation clause, omit the third real
  sentence, and end in exactly one `...` marker.
- The Markdown regression proves that behavior through the rendered Summary
  section, while JSON retains the full source summary.
- Ordinary boundaries, sentence-final abbreviation use, exact-token matching,
  UTF-8 safety, and both truncation limits satisfy the controls above.
- Focused renderer/helper tests pass, then the repository gates pass exactly:

  ```text
  make lint
  make test
  make spec
  ```

## Review

- Design review: ACCEPT (2026-09-04). The amended reproducer matches the
  current two-boundary implementation, and `deps: ["1115"]` is necessary and
  exact: 1115 must preserve the full provider summary in `Trial`/JSON while
  supplying the single Markdown-owned bounded-summary seam that 1113 changes.
  The six-form table plus ordinary-boundary, sentence-final abbreviation,
  exact-token/suffix-collision, UTF-8 byte-cap, and exactly-one-marker controls
  distinguish the intended bounded policy from fixture-only allowlisting. The
  Markdown-path assertion and full-value JSON contract keep the surface change
  scoped and compatible. With 1115 landed to that seam contract, this ticket is
  implementation-ready without provider captures, documentation changes, or a
  new parser dependency.
- Implementation evidence (2026-09-04): the failing-first renderer lane had
  the expected four failures: supported abbreviations stopped early, sentence-
  only omissions had no marker, the sentence-final/suffix controls lacked the
  marker, and the public Markdown path lost the clause after `pts.`. The
  renderer-local helper now matches only complete supported tokens ASCII-case-
  insensitively, uses the accepted lowercase/numeric lookahead plus the `Dr.`
  uppercase rule, and records sentence and byte omission before adding exactly
  one marker. A strengthened multibyte run exposed and corrected an
  implementation-order bug: token suffix matching must happen before examining
  the preceding Unicode character so an unrelated period cannot create an
  invalid UTF-8 slice. No design assumption changed. The final focused renderer
  lane passed all 12 tests; the three 1115 conversion/JSON preservation tests,
  formatting, renderer-library clippy with warnings denied, and the supported
  static specification lane (8 cases) also passed. No provider capture,
  converter, model, template, age behavior, or other renderer changed.
- Code review: ACCEPT (2026-09-04). The reviewer verified all six exact
  abbreviation forms, suffix-collision and sentence-final controls, repeated
  abbreviations, deterministic lookahead, whitespace/punctuation handling,
  honest single-marker behavior, UTF-8-safe indexing, and the 500-content-byte
  bound. Markdown is the only changed surface; the complete `Trial`/JSON value
  established by 1115 remains intact. No blocking findings remain. The
  documented `Dr.` followed by uppercase text ambiguity is the accepted
  deterministic tradeoff.

## Completed 2026-09-04

Trial Markdown summaries no longer count the periods inside `pts.`, `vs.`,
`approx.`, `e.g.`, `i.v.`, or `Dr.` as sentence boundaries under the accepted
lookahead rules. Any sentence- or byte-limited omission now ends in exactly
one `...`, while complete summaries remain unchanged and full JSON is
preserved.

Final primary gates passed on the independently accepted tree: `make lint`;
`make test`, including the complete offline Rust lane, 883 Python tests passed
and 3 skipped, and strict documentation; and `make spec`, including all routine
mustmatch pages, 38 isolation contracts, fixture cleanup, and the 8-case static
lane.
