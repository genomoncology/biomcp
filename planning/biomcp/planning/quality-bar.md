# BioMCP Quality Bar

This file records the current smoke tests, timing baselines, and known live-data fragile areas for the BioMCP repo. It folds the tracked findings from issue notes 233, 236, 283-drug-brand-name-exact-match-ranking-drift, and 283-no-quality-bar.

## Smoke Tests

- `make check` — passes. Current observed lane details from this worktree on 2026-04-26:
  - `cargo nextest run`: `1988 passed, 1 skipped` (`Summary [  19.931s] 1988 tests run: 1988 passed, 1 skipped`)
  - `uv run pytest tests/ -v --mcp-cmd "./target/release/biomcp serve"`: `234 passed in 13.75s`
  - `uv run mkdocs build --strict`: passes
  - wall time: `801.84s` (`13m21.84s` real)
- `make spec-pr` — passes. Current observed lane details from this worktree on 2026-04-26:
  - parallel entity/surface pass: `95 passed in 133.93s (0:02:13)`
  - serialized `spec/entity/protein.md` pass: `5 passed in 5.00s`
  - total current canary count: `100 passed`
  - wall time: `139.80s` (`2m19.80s` real)

## Performance Baselines

| Command | Result | Wall time | Notes |
| --- | --- | --- | --- |
| `make check` | pass | `801.84s` | Includes `cargo build --release --locked` for `test-contracts`; this run compiled the release binary in `5m36s` before the Python/docs contract lane. |
| `make spec-pr` | `100 passed` | `139.80s` | Observed after `make check`; the main xdist leg carried `95` specs and the serialized protein leg carried `5`. |
| `make release-gate` warm budget | target | `<=5m` | The governing budget is the warm cached spec leg documented in `spec/README-timings.md`; the active requirement is to keep the release-gate canary portion under five minutes warm and under fifteen minutes cold per cache key. |

## Fragile Areas

- **Monarch Initiative / OLS4 disease relationship labels and IDs** (`233`)
  - **Upstream system:** Monarch disease-gene relationships plus OLS4-normalized disease follow-up identifiers.
  - **Symptom:** live label drift changes rows such as `SNCA -> Parkinson disease` from `causes` to `gene associated with condition`, and OLS4 normalization can change `next_commands` IDs. Exact `mustmatch` assertions in disease specs then fail even though the biological relationship is still present.
  - **Mitigation:** prefer semantic checks on the returned gene/disease presence or accepted label variants instead of pinning one source-specific relationship label or one normalized follow-up command literal. If a live drift blocks the gate, triage it as a dedicated follow-up rather than broadening this quality bar.
- **NCBI GTR bulk export `test_type` schema drift** (`236`)
  - **Upstream system:** NCBI GTR downloaded diagnostic-test bundle (`test_version.gz`, `test_condition_gene.txt`).
  - **Symptom:** the upstream export dropped the old `test_type` column from `test_version.gz` and the live values no longer include stale examples such as `molecular`; hard-coded live smoke examples can stop matching real data.
  - **Mitigation:** use currently shipped values from the synced bundle, or derive smoke examples from the downloaded data instead of pinning retired labels. The runtime already backfills `test_type` from `test_condition_gene.txt`, so planning/proof artifacts must stay aligned with that contract.
- **OpenFDA brand-name rescue ranking for combination products** (`283-drug-brand-name-exact-match-ranking-drift`)
  - **Upstream system:** OpenFDA drug-label and brand-name rescue results.
  - **Symptom:** exact-match searches such as `Keytruda` can surface the combination product `KEYTRUDA QLEX` alongside or ahead of the base product, which breaks exact negative assertions like `mustmatch not like "pembrolizumab and berahyaluronidase alfa-pmph"` in the drug canary.
  - **Mitigation:** treat single-row ranking assumptions as fragile until the rescue ordering is intentionally specified. Keep the failure documented, and when updating proof prefer assertions about the expected base product being present rather than assuming the combination product can never appear.
- **Historical live-network spec suppressions (`SPEC_PR_DESELECT_ARGS`)** (`283-no-quality-bar`)
  - **Upstream system:** mixed live article/network dependencies behind the old numbered spec corpus, especially PubMed-family lookups and other latency-sensitive online assertions.
  - **Symptom:** the old corpus used heading-coupled `SPEC_PR_DESELECT_ARGS` suppressions when live-network headings timed out or drifted. That masked regressions, and the suppressions themselves broke when headings were renamed.
  - **Mitigation:** the active v2 canary lane removes permanent `SPEC_PR_DESELECT_ARGS` dependence. Keep live-network assertions semantic, rely on the cache-backed `tools/biomcp-ci` / `make spec-pr` path, and file explicit drift tickets instead of reintroducing a standing deselect list.

## Last Updated

2026-04-26
