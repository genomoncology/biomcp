# Code Review — ticket 679

## Scope and traceability

Reviewed `main..HEAD`, ticket/design artifacts, receipt record, code log, full runtime and documentation diffs, capture, fixtures, and changed lifecycle tests.

- Design completeness: every named classifier, fallback, projection, capture, fixture, documentation, and test change landed.
- Forward traceability: all six proof-matrix entries land at their stated locations.
- Reverse traceability: `git diff main..HEAD -- 'spec/*'` contains only the designed routine PoW assertion and live HTML/XHTML strengthening. No shipped assertion was invented, removed, relaxed, or made trivia-only.
- Edit discipline: the implementation is within the designed source-classifier/projection slice. Fixture-workspace capture copies are required consequences of the fixture now loading the receipted capture. No over-edit found.
- Independent review noted that a PoW coverage row can coexist with a differently identified successful asset. This is not a defect: reconciliation is by canonical provider identity, and different identities may legitimately name different same-filename objects. Within a canonical identity, `fetch_first_available_with_limit` returns bytes immediately and suppresses the PoW pending coverage.

## Repair

Added a small `is_pmc_proof_of_work` classifier and a native unit test proving both required markers are case-insensitive. The pre-review capture-only test used their observed casing, so it would not detect a regression from case-insensitive matching to exact matching. No mustmatch spec was changed.

Collateral scan after the repair found no dead branches/imports, cleanup conflicts, stale errors, or shadowed variables.

## Validation

- Before repair: `make lint`, `make test` (Rust suite, 448 Python contracts, strict MkDocs), and `make spec`: passed.
- After repair: `cargo test --locked sources::pmc_article::tests --no-fail-fast` (9 passed), `make spec`, `make lint`, and `git diff --check`: passed.
- `make verify` remains intentionally operator-pending, as recorded by the final design; it calls real PMC and is outside the routine gate.

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| 1 | weak-assertion | no | The source-classification proof used only the receipted markers' original casing, leaving the required case-insensitive detection behavior unprotected. Repaired with direct native coverage for both markers. |

## Residual concerns

No out-of-scope issue filed.
