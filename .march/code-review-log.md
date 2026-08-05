# Code Review — ticket 678

## Scope and traceability

Reviewed `main..HEAD`, the design draft/final, red-check record, code log, fixtures, captures, docs, and all changed runtime paths.

- Design completeness: every named implementation, documentation, fixture, capture, policy, and health item has a landed change.
- Forward traceability: all five proof-matrix assertions land at their named unit, mustmatch, or ignored-live locations.
- Reverse traceability: `git diff main..HEAD -- 'spec/*'` changed only the two package URL assertions. The full-text assertion is in the proof matrix; the assets assertion replacement is expressly authorized by the operator ruling in `.march/ticket.md`. No shipped assertion was invented, weakened, or silently removed.
- Edit discipline: 538 additions / 460 deletions include the design-named tar-to-object migration and capture-receipt ordering. No excess runtime edit remained.

## Repairs

- Routed list/metadata HTTP failures and provider-policy rejections through the PMC OA package-route failure path, and made that path project as a specific public error rather than generic PMC API/provider failure.
- Required S3 listing and metadata identities to match the requested PMCID and each other, preventing a valid-but-wrong provider object from becoming article provenance.
- Added bounded native regression coverage for the public error projection and PMCID identity check; updated the affected source documentation.

## Validation

- `cargo test --locked sources::pmc_oa::tests --no-fail-fast`: passed (16 passed, 1 ignored)
- `make lint`: passed
- `make test`: passed (448 Python contracts; Rust tests; strict MkDocs build)
- `make spec`: passed (90 passed/3 skipped, 220 passed/2 skipped, 7 passed, 10 static passed)
- `git diff --check`: passed

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| 1 | error-classification | no | S3 listing and metadata HTTP failures were generic `Api` errors, and the existing route error was publicly projected as a generic PMC provider failure, contrary to the required package-route attribution. Repaired. |
| 2 | validation-gap | no | A syntactically valid S3 listing or metadata object could identify a different PMCID than the request; the client would fetch and expose that other article's provenance. Repaired with identity checks. |

## Residual concerns

None. No out-of-scope issue was identified.
