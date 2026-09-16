---
flow: build
priority: 1
deps: []
---

# 1198: The trial search stops escaping hyphens and reports verification-emptied zeros

## Goal

`biomcp search trial --source ctgov` sends hyphenated search terms to
ClinicalTrials.gov unescaped, so `--criteria "prior anti-PD-1 therapy"` matches
what the registry actually holds; and when eligibility post-verification removes
every provider match, the zero output names the upstream count instead of
implying no trials exist.

## Current Facts

`essie_escape` (`src/entities/trial/search/essie.rs:10`) backslash-escapes the
ASCII hyphen (`-` → `\-`). ClinicalTrials.gov's ESSIE query parser treats the
escaped form as a different, far narrower phrase. Measured 2026-09-16 against
the live v2 API with `query.term=AREA[EligibilityCriteria](...)`:

| phrase (quoted) | plain | escaped |
| --- | --- | --- |
| `"prior anti-PD-1 therapy"` | 24 | 1 |
| `"anti-PD-1 therapy"` | 137 | 2 |
| `"anti-PD-1"` | 4721 | 49 |
| `"PD-1"` | 7629 | 201 |

The same defect hits `query.intr` quoted literals: `"anti-PD-1"` 1644 → 11,
`"PD-1"` 4383 → 48. Every CTGov ESSIE surface funnels through the same escape:
`--criteria`, `--mutation`, `--biomarker`, `--sponsor`, `--study-type`
(`ctgov.rs:131-173`), `--prior-therapies`, `--progression-on`
(`essie.rs:158-183`), and the quoted intervention literal (`ctgov.rs:233`).
`--source nci` passes structured parameters and is unaffected.

Escaping is behavior-neutral for every other escaped character measured
(`+`, `(`, `)`, `:`, `/` returned identical counts escaped and unescaped), and
unescaped hyphens never errored in any tested position (leading, trailing,
doubled, spaced, mid-word, before letters and before digits):
`"-mutant BRAF"` 7, `"BRAF--wildtype"` 161, `"BRAF - V600E"` 431,
`"ECOG 0-"` 13421, `"well-differentiated"` 1766, `"CAR-T"` 2453.

Second symptom: when eligibility verification
(`eligibility_keyword_in_inclusion`, `src/entities/trial/search/eligibility.rs`)
removes every provider match, the command prints
`No trials found matching the filters.` with no indication that the provider
returned rows. The provider total (`state.total` from `totalCount`) is
discarded by `finish_ctgov_single_page` (`ctgov.rs:515-535`) because
`returned_total` counts only verified rows. The original observation is
`sdlc/issues/2026-09-11-search-trial-criteria-returns-zero-results.md`.

## Design

### Escape set

`essie_escape` escapes exactly these characters after this ticket: backslash,
double quote, plus, exclamation, open and close parenthesis, braces, brackets,
caret, tilde, star, question mark, colon, slash, and pipe. The hyphen
(`-`) is removed from the set and every other character and behavior stays
byte-identical. The rendered term for
`--criteria "anti-PD-1 therapy"` is exactly
`AREA[EligibilityCriteria]("anti-PD-1 therapy")`; for the quoted intervention
literal, exactly `"anti-PD-1"`. No other query grammar changes: boolean-operator
splitting, term quoting, and term joining are untouched.

### Honest zero

- `SearchPage<T>` (`src/entities/mod.rs:32`) gains
  `pub upstream_total: Option<usize>`. The existing `offset` and `cursor`
  constructors set `None`; a new
  `cursor_with_upstream(results, total, next_page_token, upstream_total)`
  carries the value.
- `finish_ctgov_single_page` (`ctgov.rs`) fills
  `upstream_total = state.total` exactly when the verification that ran was
  eligibility verification (`context.eligibility_keywords` non-empty); a
  facility-geo-only verification and every other call site leave `None`. This
  keeps the hint's wording true: it is about eligibility text, not geography.
- In the CLI trial dispatch (`src/cli/trial/dispatch.rs`), when
  `results.is_empty()` and `page.upstream_total == Some(n)` with `n > 0`:
  - Markdown: the first bullet of the existing "Try broadening the filtered
    search:" block is exactly this string (single spaces, one line):

    ```text
    ClinicalTrials.gov matched {n} trial(s) on this eligibility text, but registry eligibility verification removed all of them (the term appears only in exclusion criteria or outside the inclusion section). Try a shorter phrase or `--mutation` for broader field coverage.
    ```
  - JSON: a new optional `_meta.upstream_total` member carrying `n`, plus the
    existing `zero_result_trial_next_commands` relaxed command when
    `--criteria` was set: the same filters with `criteria = None` and
    `mutation = <criteria value>`, rendered by the existing
    `trial_search_command` builder. (The current mutation-relaxation command
    stays; this adds the criteria-relaxation command.)
- `SearchJsonMeta` (`src/cli/shared.rs:541`) gains
  `#[serde(skip_serializing_if = "Option::is_none")] pub upstream_total: Option<usize>`,
  populated only on this trial path; every other search JSON is unchanged.
  The value is present only for the verification-emptied zero case; a normal
  empty search carries no member.

## Fixtures

Focused tests:

1. `essie.rs` — a table test pins `essie_escape` output for every character in
   the escape set plus representatives: `-`, `+`, `(`, `)`, `:`, `/`, `\`,
   `"`. Exact expectations include
   `essie_escape("anti-PD-1") == "anti-PD-1"`,
   `essie_escape("PD-L1") == "PD-L1"`,
   `essie_escape("3L+") == "3L\\+"`,
   `essie_escape_boolean_expression("dMMR OR PD-1") == "\"dMMR\" OR \"PD-1\""`.
   The existing `criteria_query_and_verification_share_case_sensitive_operator_rules`
   corpus is unchanged (it contains no hyphens).
2. `ctgov.rs` — `ctgov_query_term` pins the rendered string for
   `--criteria "anti-PD-1 therapy"` and for a quoted intervention literal of
   `anti-PD-1` on `--source ctgov`.
3. Executable spec (`spec/entity/trial.md` plus the existing
   `setup-ctgov-intervention-alias-spec-fixture.sh` fixture family — do not
   create a second trial fixture family):
   - Add a fixture route: a search whose quoted eligibility phrase contains
     `anti-PD-1` returns one study with `totalCount: 1`; the spec runs
     `biomcp search trial --criteria "anti-PD-1 therapy"` and asserts the
     run: (a) returns that trial, and (b) the request log line contains
     `anti-PD-1+therapy` and does not contain `%5C-` (the escaped form fails
     the route and the log assertion).
   - Add a fixture route: `--criteria "verification-emptied-fixture"` returns
     two studies whose `eligibilityModule.eligibilityCriteria` place the term
     only under `Exclusion Criteria:` with `totalCount: 2`; detail fetches
     return the same module. Two spec blocks pin: (a) the exact Markdown hint
     bullet for `n = 2`, and (b) the JSON `_meta.upstream_total == 2` with
     `count == 0` and the relaxed `--mutation` next command present.

## Acceptance

The escape set no longer contains `-`; the hyphen table and the query-term pins
pass; the two spec blocks pass; `make lint`, `make test`, and `make spec` pass
on the gate host at the pushed SHA. The package path count is unchanged. No
other trial search behavior changes: non-hyphen terms render byte-identically
to today, and every existing test that pins escaped strings remains valid
except where it deliberately pinned the hyphen.

## Dependencies

None.

## Boundaries

No change to `essie_escape`'s other characters, to boolean-expression
splitting, or to the term quoting grammar. No change to the eligibility
verification rules themselves (`eligibility_keyword_in_inclusion`) or to which
matches they remove. No change to `--source nci` (structured parameters, no
ESSIE escaping). No hint or count change on the `--count-only` path: its
verified-count semantics stay exactly as they are. No new CLI flag. No change
to the `--mutation` broad-discovery field set.

## Complexity

- Contract score: 1 (several explicit public cases: the escape table, the query
  terms, the zero-output message and JSON member)
- State and timing score: 0 (query construction and output; no persistent
  state)
- Reach score: 1 (trial search modules, the CLI zero-result path, and the
  search-JSON envelope)
- Proof score: 1 (unit tables, request-log pins, and executable spec blocks;
  no hostile-input tier)
- Cost of error score: 1 (a wrong recall or a wrong zero is user-visible; no
  data loss)
- Total: 4
- Minimum level floor: none
- Final level: 2
- Reasons: several explicit query surfaces through one shared escape function
  with request-shape and output proofs; no state, concurrency, or
  compatibility decision
- Selected model: gpt-5.6-luna, high reasoning (level 2 implementer)

## Review

- Design review: pending
- Code review: pending
