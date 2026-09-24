---
base: 1a6a7f1e
status: in progress, not landed
---

Work stopped for the night on 2026-09-23. The branch is `biodata/2003-search-patients`.

## Done

- `bd7be090` folds the design review's four low findings into the ticket.
- `5139ddfa` (red) adds the tests with the search guards stubbed out. `7cda0b57` (green) turns the guards on. A later commit drops an unused re-export that clippy refused.
- Code: `FhirClient::read_metadata` and `FhirClient::search_patients` build on the 2002 request function, which now takes a strict flag for `Prefer: handling=strict`. `src/entities/patient/search.rs` checks values, reads `metadata`, refuses an undeclared parameter, and keeps only id, gender, and birth date. The CLI, list page, help, docs, changelog, spec page, and fixture routes cover search.

## Evidence on yellow (under the gate lock)

- Red at `5139ddfa`: unit 33 of 40 passed, 7 failed (every new search test). Contract 9 of 12 passed, 3 failed (bad values, trimmed output, stdio search).
- Green at `7cda0b57`: unit 40 of 40, contract 12 of 12.
- `cargo fmt --check` failed with 18 hunks. Clippy failed on the unused re-export, fixed here and not yet rechecked.

## Next step

1. On yellow, run `cargo fmt` at the tip and bring the result back. Rust formatting never runs on the development box.
2. Run `make lint`, `make test`, `make spec`, and `tools/run-offline -- tools/check-biodata-1.0 --already-isolated` on yellow at the pushed SHA.
3. Run the live smoke against the blue demo HAPI server. A read-only curl on 2026-09-23 found `gender`, `birthdate`, and `_has` declared on the server Patient resource. The smoke must still run the strict `--condition` case with the built binary.
4. Hand to code review. Do not land before review.

## Open

- The metadata refusal names the first missing parameter in send order and not all of them.
- JSON rows use `birth_date` to match `get patient`. The ticket names the FHIR field `birthDate`.
