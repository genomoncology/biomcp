# Code Review — ticket 662

## Scope and traceability

Reviewed the complete `main..HEAD` diff (17 files; 810 additions / 85 deletions before review repairs), the final design, red-check record, and code log. The data-heavy diff is proportionate to seven raw captured responses plus receipts. The only runtime edit not named by the design was reverted.

Forward traceability: all four proof-matrix entries landed at their named locations. Reverse traceability used `git diff main..HEAD -- 'spec/*'`: the new live-lane ratchet is explicitly named in the matrix, and deletion of exactly the two named live pages is authorized. No invented, relaxed, or silently removed shipped assertion remains.

## Repairs

- Extended CAR’s receipt-backed decoder test to consume the recorded empty and malformed response bytes, pinning their distinct non-resolved outcomes.
- Added LDH raw-capture coverage through the production medium and direct clients, plus the malformed direct verification outcome.
- Reverted the undesigned change that treated an LDH annotation with no body as complete. The real direct capture legitimately contains an unrelated body-less annotation; it may still yield the required linkage while the overall result remains incomplete.

## Validation

- Focused CAR/LDH receipt-backed tests: passed
- `uv run --no-sync pytest tests/test_capture_receipts.py -k clingen_car_and_ldh_live_replacements_have_receipted_captures -v`: passed
- `uv run --no-sync python tools/check-source-capture-receipts.py --root testdata/sources --json`: passed (107 classified, 0 byte-unfaithful)
- `make spec`: passed
- `make lint`: passed
- `make test`: passed (448 Python contracts; strict MkDocs build)
- `git diff --check`: passed

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| 1 | missing-assertion | no | CAR empty/malformed and LDH medium/empty/malformed captures were receipt-admitted but not consumed by production-path tests, leaving required captured-outcome coverage absent. Repaired with bounded native tests. |
| 2 | over-edit | no | `verify_ldh_annotation` stopped marking body-less annotations incomplete solely so the positive capture could assert completeness. This was an undesigned runtime behavior change. Reverted; the linkage assertion remains. |

## Residual concerns

None. No out-of-scope issue was identified.
