# Harden — 369 Score variant-agent source APIs

## Decomposition

This BioMCP spike is a research/experiment ticket, not a Zig or production-runtime implementation. The ticket explicitly kept runtime connectors and CLI/MCP behavior out of scope, so hardening decomposed the optimized experiment harness rather than touching production BioMCP source.

Extracted reusable library package:

- `architecture/experiments/369-score-variant-agent-source-apis/lib/source_api_scoring/types.py`
  - shared immutable probe records: `CommandProbe`, `HttpProbe`.
- `architecture/experiments/369-score-variant-agent-source-apis/lib/source_api_scoring/current_biomcp.py`
  - current BioMCP CLI probe catalog, command execution, and JSON summarization.
- `architecture/experiments/369-score-variant-agent-source-apis/lib/source_api_scoring/external_apis.py`
  - public HTTP probe catalog, request execution, JSON path helper, and source-specific payload summaries.
- `architecture/experiments/369-score-variant-agent-source-apis/lib/source_api_scoring/feasibility.py`
  - scoring criteria, candidate source matrix rows, BioMCP/agent boundary classifications, follow-up recommendations, matrix builder/writer.
- `architecture/experiments/369-score-variant-agent-source-apis/lib/source_api_scoring/__init__.py`
  - stable import facade for downstream spike scripts.

Thin CLI wrappers retained for convenience only:

- `scripts/probe_current_biomcp.py` — 31 lines.
- `scripts/probe_external_apis.py` — 27 lines.
- `scripts/synthesize_feasibility_matrix.py` — 28 lines.

The wrappers now do only argument parsing, local import-path bootstrapping, output path selection, and one-line status printing. Probe definitions, algorithms, summaries, concurrency, and matrix synthesis live in the library.

## Public API

Import package: `source_api_scoring` from:

```text
architecture/experiments/369-score-variant-agent-source-apis/lib/
```

Public types:

- `CommandProbe(group, label, args, expect_json=True)`
- `HttpProbe(group, service, label, method, url, body=None, headers=None)`

Current BioMCP probe API:

- `default_command_probes() -> list[CommandProbe]`
- `summarize_biomcp_json(value) -> dict`
- `run_command_probe(bin_path, probe) -> dict`
- `run_current_biomcp_suite(bin_path, probes=None, max_workers=8) -> dict`
- `write_current_biomcp_report(path, bin_path, probes=None, max_workers=8) -> dict`

External source probe API:

- `default_http_probes() -> list[HttpProbe]`
- `json_at(value, path) -> Any`
- `summarize_http_payload(service, payload) -> dict`
- `request_http_probe(probe, timeout=20) -> dict`
- `run_external_api_suite(probes=None, max_workers=6, timeout=20) -> dict`
- `write_external_api_report(path, probes=None, max_workers=6, timeout=20) -> dict`

Feasibility/scoring API:

- `CRITERIA`
- `CANDIDATES`
- `BOUNDARY_CLASSIFICATIONS`
- `FOLLOW_UP_RECOMMENDATIONS`
- `load_probe_index(path) -> dict[str, list[dict]]`
- `build_feasibility_matrix(external_probe_report_or_index) -> dict`
- `write_feasibility_matrix(path, external_path) -> dict`

Usage example for a downstream spike that wants to reuse the normalization probes without shelling out:

```python
from source_api_scoring import default_http_probes, run_external_api_suite

normalization_probes = [
    probe for probe in default_http_probes()
    if probe.group == "normalization"
]
report = run_external_api_suite(normalization_probes, max_workers=4)
print(report["probe_count"])
```

Usage example for a downstream capability-discovery spike that wants the source matrix and ordered follow-up recommendations:

```python
from source_api_scoring import build_feasibility_matrix, run_external_api_suite

external_report = run_external_api_suite()
matrix = build_feasibility_matrix(external_report)
for item in matrix["recommended_follow_up_tickets"]:
    print(item["order"], item["title"])
```

Usage example for a downstream exact-population spike that wants the gnomAD/MyVariant probe rows only:

```python
from source_api_scoring import default_http_probes, run_external_api_suite

population_inputs = [
    probe for probe in default_http_probes()
    if probe.group == "population" or probe.service == "myvariant"
]
report = run_external_api_suite(population_inputs, max_workers=3)
```

This API is intentionally experiment-scoped. Production BioMCP connectors should still be implemented in Rust source modules with normal source provenance, health/list/docs/spec alignment, and no dependency on experiment Python at runtime.

## Build System

There is no `build.zig` in this worktree; BioMCP is a Rust project plus experiment Python scripts. The ticket also says no production repo code should be modified. Therefore no Cargo/build-system change was appropriate.

To depend on this spike library from another experiment/downstream spike script, add the experiment-local library directory to `PYTHONPATH`:

```bash
PYTHONPATH=architecture/experiments/369-score-variant-agent-source-apis/lib \
  uv run --no-project --python 3.12 python downstream_spike.py
```

Or, for a wrapper located inside `architecture/experiments/369-score-variant-agent-source-apis/scripts/`, use the established wrapper pattern:

```python
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "lib"))
from source_api_scoring import build_feasibility_matrix
```

Downstream March tickets that implement production behavior should copy the decision contracts into Rust designs/tests, not call the experiment package from runtime BioMCP.

## Regression Check

Benchmark suite rerun through the refactored thin wrappers:

```bash
architecture/experiments/369-score-variant-agent-source-apis/scripts/probe_current_biomcp.py \
  --out .march/harden-bench/harden_final_current_biomcp.json
architecture/experiments/369-score-variant-agent-source-apis/scripts/probe_external_apis.py \
  --out .march/harden-bench/harden_final_external_api_probes.json
architecture/experiments/369-score-variant-agent-source-apis/scripts/synthesize_feasibility_matrix.py \
  --external .march/harden-bench/harden_final_external_api_probes.json \
  --out .march/harden-bench/harden_final_source_feasibility_matrix.json
```

Best harden rerun timings:

| Component | Harden result | Harden wall time |
|---|---:|---:|
| Current BioMCP CLI probes | 11 probes; expected return codes preserved | 2.050 s |
| External API probes | 18 probes; expected statuses preserved (`ensembl_vep` expected HTTP 400) | 1.053 s |
| Source feasibility matrix synthesis | 21 candidates | 0.070 s |
| End-to-end suite | contract counts and status/shape preserved | 3.173 s |

Contract summary:

- Current probe count: 11.
- External probe count: 18.
- Candidate source count: 21.
- Classification counts: 12 good BioMCP proxy candidates / 8 possible-but-gated / 1 reject/default-exclude.
- Boundary rows: 6.
- Follow-up recommendations: 4.
- Expected current-surface failures preserved:
  - `MITF transcript HGVS unsupported` returned code 2.
  - `ASCO DOI via current get` returned code 1.
- External probes preserved expected status shape: all OK except `ensembl_vep` expected HTTP 400 for RefSeq transcript HGVS.

The refactor preserves the optimized concurrency settings (`max_workers=8` for current BioMCP probes, `max_workers=6` for public HTTP probes) and beat the optimized final suite number recorded in `.march/optimize.md` (6.38 s). Earlier harden runs showed public-network jitter, but no status/shape regression.

Validation suite:

- Python import smoke with `PYTHONPATH=.../lib uv run --no-project --python 3.12 python`: passed; 18 default HTTP probes, 21 matrix candidates, 6 boundary rows, 4 follow-ups.
- `~/workspace/scripts/lint-planning.sh biomcp`: passed (`all clean`).
- `cargo test --workspace --all-targets`: passed; 1965 lib tests plus integration suites, 0 failures, one pre-existing ignored EMA test.

## Reusable Assets

Downstream spikes inherit these concrete assets:

- Typed probe records (`CommandProbe`, `HttpProbe`).
- Current BioMCP regression probe catalog for variant/article/list surfaces.
- Public source probe catalog covering normalization, population, literature, DOI metadata, access status, and curated-source evidence candidates.
- Source-specific JSON summarizers for Mutalyzer, VariantValidator, NCBI SPDI, ClinGen Allele Registry, MyVariant, Ensembl VEP, gnomAD, PubMed, Europe PMC, PubTator3, LitSense2, Semantic Scholar, Crossref, OpenAlex, and Unpaywall.
- Feasibility scoring criteria for source/API selection.
- 21-row candidate source matrix constants.
- BioMCP-vs-agent boundary classifications.
- Four ordered follow-up recommendation records.
- Benchmark/validation wrapper pattern for experiment scripts that import library code rather than copying or shelling out.
