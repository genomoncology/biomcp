# Code Review — Ticket 647

## Scope reviewed

- Read `AGENTS.md`, ticket, design draft/final, red-check evidence, and code log.
- Reviewed the full `main..HEAD` diff and the complete changed capture-store implementation.
- Checked design completeness, proof-matrix traceability, shipped-spec diff, edit discipline, security, duplication, and post-fix collateral damage.

## Audit results

- **Design completeness:** all descriptor-relative traversal, publication, reads, scans, cleanup, lock handling, Unix/non-Unix boundary, documentation, and controlled swap proof work landed.
- **Traceability:** the sole proof-matrix assertion landed at `src/cache/provider_capture.rs::refuses_a_blob_shard_swapped_after_validation_without_writing_outside`. `git diff main..HEAD -- 'spec/*'` is empty; no shipped assertion was invented, relaxed, or removed.
- **Edit discipline:** the 1,031-line capture-store change is proportionate to replacing all named managed-tree path operations with descriptor-relative equivalents. The documentation and Windows compile-only CI adjustment are named/mechanical platform-boundary consequences. No over-edit found.
- **Security/quality:** no path, shell, query, secret, or authorization issue found. Descriptor handles are RAII-owned; no dead code, unused import, double cleanup, stale error message, or shadowed variable resulted from the review repair.

## Fix applied

`CaptureDirectory::read_file` had treated every failed no-follow open whose subsequent `file_status` returned `None` as absent. `file_status` uses `None` for both a missing entry and a directory, so an attacker or corruption replacing an expected metadata file with a directory was reported as `Unavailable` rather than `Corrupt`. The repair checks specifically for `ENOENT`; any extant non-regular entry remains corrupt. A narrow Unix native regression test covers the metadata-directory case.

## Validation

- `cargo nextest run cache::provider_capture --no-fail-fast` — pass (14 tests)
- `make lint` — pass
- `make test` — pass (446 Python tests; native and docs-contract lanes)
- `make spec` — pass (90 passed/3 skipped; 218 passed/2 skipped; 7 passed; 31 Python contracts; 10 static specs)
- `git diff --check` — pass

## Residual concerns

None within ticket scope. No out-of-scope issue filed.

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| 1 | error-classification | no | A directory replacing an expected capture metadata file was misclassified as absent (`Unavailable`) rather than corrupt; descriptor-relative reads now map only `ENOENT` to absence. |
