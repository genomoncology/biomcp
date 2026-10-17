# 2160 — Isolate the two unstable MCP provider contracts

Status: OPEN.

Milestone: 0.9.3

## Outcome

The FAERS report-share and raw/typed article-query contracts return their recorded fixture outcomes independently of live providers, credentials and unrelated test processes.

- Starts from: `tests/rmcp_client_contract.rs`, shared MCP fixture setup and job `1792239358938-20043` on main `2bc79be`. The previous routine candidate run passed; the main run failed two contracts without changes to their product behavior.
- Keeps: the existing report-share wording and raw/typed query equivalence assertions; actual provider outages still remain named outages in product output. Do not weaken assertions or add a broad fixture framework.
- Changes: claims `tests/rmcp_client_contract.rs`, `crates/biomcp-mcp-contract-client/src/lib.rs` and `CHANGELOG.md`. The shared harness must give each spawned server an isolated home, configuration, cache and provider environment; each contract can explicitly supply its own fixture settings. Narrow the observed cause before implementation.
- Proof: reproduce the observed failure or its identified environmental trigger on current main through the build job before implementation. Pin the actual requests with existing isolated fixtures, then run both existing contracts and their affected siblings successfully. The complete routine gate must pass. Do not print credential values or derive them for records.
- Defers: broad fixture identifiers, replay migration and unrelated provider policy changes to their existing tickets or the next release.

## Reproduction

Job `1792239358938-20043` ran `pm job run 2154 2bc79be -- bash -c 'unset CARGO_INCREMENTAL; make lint && make test && make spec && make full-feature-check'` on current main and exited 2 during Rust tests. `raw_mcp_preserves_faers_report_share_context_in_json_and_markdown` failed at its `Share of returned reports` assertion. `raw_and_typed_article_query_validation_converges_before_provider_work` compared an unavailable Semantic Scholar answer against a complete empty answer. The same unchanged contracts passed the preceding candidate run, so isolate the concrete input or fixture cause rather than accepting a blind rerun.

PM decision: 2026-10-17. Ian approved narrow repairs that block main's checks before ordinary landings. Existing source facts and product decisions stay unchanged; report any new clinical interpretation for clinical review.

### Controlled environment reproduction

The baseline is main `078ce756d595d09a58262c888fac40a0893470cc`. Before implementation, job `1792240909659-39024` ran the two existing contracts with a valid synthetic `S2_API_KEY` and passed. Job `1792240976513-50844` then ran this command and exited 100:

```sh
pm job run 2160 078ce756d -- python3 -c 'import os,subprocess; os.environ["S2_API_KEY"]="synthetic\ncontract-key"; raise SystemExit(subprocess.call(["cargo","nextest","run","--locked","--no-default-features","--test","rmcp_client_contract","-E","test(raw_and_typed_article_query_validation_converges_before_provider_work)"]))'
```

The unchanged contract failed at its request-count assertion: `left: 0`, `right: 2`. An inherited malformed synthetic authentication header prevents the fixture requests. This establishes an environment-isolation defect. It does not establish that the historical FAERS failure had this trigger; that failure's exact trigger remains unknown. The shared harness must exclude inherited credentials and product configuration while retaining explicit per-contract environment overrides.
