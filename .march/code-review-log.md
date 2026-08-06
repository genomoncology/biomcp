# Code Review — Ticket 680

## Scope reviewed

- Read `AGENTS.md`; confirmed shipped behavioral specs are `spec/*.md`, run by `make spec`.
- Read the ticket, design draft/final, red-check record, and code log.
- Reviewed the full `main..HEAD` diff and the pending implementation diff against `main`.
- Reviewed the CVX constructor/root-resolution call sites, the VAERS resolver call graph, the temporary-directory guard, and the checked-in CVX fixture data.

## Design completeness

All final-design items are implemented:

1. `CvxClient::from_root` remains test-only and is now `pub(crate)` for the adverse-event test module.
2. The resolver's normalized-query/candidate decision branch is shared by production and test-only callers.
3. Production `resolve_vaers_vaccine` still obtains candidates through the unchanged LocalOnly (`CvxClient::new`) and AutoSync (`CvxClient::ready(CvxSyncMode::Auto)`) branches.
4. The test-only root helper parses real fixture files with `CvxClient::from_root` and does not mutate environment state.
5. The three named tests each create an isolated `TempDirGuard` root containing all required CVX files. Their outcome assertions are unchanged.
6. The constructor/root sweep found no additional affected test caller.

## Spec traceability

The proof matrix deliberately contains three native-unit entries and no mustmatch entry because the seam is internal and creates no shipped behavior.

- **Forward:** all three named proof nodes remain in `src/entities/adverse_event.rs`; their setup change is present in the implementation diff and their semantic outcome assertions remain intact.
- **Reverse:** both `git diff main..HEAD -- 'spec/*'` and the pending `spec/*` diff are empty. No shipped assertion was added, relaxed, or removed. No docs are changed.
- No behavior assertion was placed in the wrong layer.

## Edit discipline and quality

The pending implementation changes 86 lines in two files (70 additions, 16 deletions). The minimal planned shape is about 70 lines: constructor visibility, the 17-line shared resolver extraction, the test-only helper, a three-file fixture writer, and three setup substitutions. This is well below the 3x scrutiny threshold. Every changed category is named by the final design or a direct mechanical consequence.

The shared resolver preserves the prior decision order and result text. The test helper remains behind `#[cfg(test)]`, the temporary roots are RAII-managed, fixture writes use fixed checked-in bytes, and no untrusted value reaches a shell, path outside the temporary root, network call, or global environment. The local fixture writer duplicates no production behavior; the only related writer is private to the CVX source test submodule and cannot be reused from this consumer test module without widening unrelated test support.

Post-review collateral scan found no dead code, unused import/variable, duplicate cleanup, stale error, or variable shadowing introduced by the change.

## Validation

- `env -u BIOMCP_CVX_DIR XDG_DATA_HOME=$(mktemp -d) cargo nextest run --locked` for each named test: 3/3 passed.
- Same isolated-data-root environment, `cargo nextest run --locked entities::adverse_event::tests`: 31/31 passed.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `make lint`: passed.
- `make test`: passed (native tests, 448 Python contracts, strict MkDocs build).
- `make spec`: passed (318 mustmatch assertions passed, 5 skipped; 31 surface checks; 10 static checks).

## Residual concerns and issues

None. The separate reviewer subagent did not return before its timeout; the primary review completed the independent source, traceability, and gate checks above. No out-of-scope issue was identified.

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| — | None found | — | No defects found; no repair or separate code-review commit required. |
