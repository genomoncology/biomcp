# Harden — Spike source-first article Markdown coverage

## Decomposition

This ticket is a BioMCP research/experiment spike, not a Zig production implementation. The prompt's `~/workspace/planning/mole/spike-plan.md` file is absent, and `.march/ticket.md` is team `biomcp`, repo `biomcp`. The reusable artifact is therefore the optimized offline source-coverage analyzer over persisted probe JSON; BioMCP runtime behavior remains unchanged.

Extracted library package:

- `architecture/experiments/381-spike-source-first-article-markdown-coverage/scripts/source_coverage_analysis/`
  - `constants.py` — stable source-family, quality-flag, and count-key definitions.
  - `model.py` — `JsonObject`, `Row`, and `Summary` type aliases for downstream analysis scripts.
  - `rows.py` — source availability, license extraction, quality bit extraction, and case/source row normalization.
  - `summary.py` — source-family summaries, contract numbers, and decision text.
  - `compare.py` — baseline/current regression comparison over rows or probe JSON.
  - `io.py` — JSON/CSV read/write helpers with stable artifact formatting.
  - `cli.py` — argument parsing and file orchestration only.
  - `__init__.py` — import facade for downstream spikes.

Thin CLI wrapper:

- `architecture/experiments/381-spike-source-first-article-markdown-coverage/scripts/analyze_source_coverage.py`
- 16 lines after extraction.
- Only keeps the uv shebang/script metadata and delegates to `source_coverage_analysis.cli.main()`.

Why this split:

- Downstream spikes should import row normalization, contract numbers, and comparison helpers directly instead of shelling out or copy-pasting.
- The CLI remains a convenience for regenerating this spike's JSON/CSV artifacts.
- No production Rust code, Cargo metadata, or BioMCP runtime source was changed because ticket 381 explicitly scoped this as a no-runtime-change spike.

Downstream consumers from the BioMCP planning issue/frontier:

1. BioC-to-Markdown renderer build — compare renderer fixture metrics against this spike's BioC/JATS counts.
2. BioC/PubTator article fulltext source rung build — reuse source-family naming, availability semantics, and regression comparison.
3. Article Markdown harvest/batch mode — reuse row schema, contract numbers, and CSV/JSON writers for manifest-like proof.
4. Source-quality flags build/architect work — reuse `QUALITY_KEYS`, `COUNT_KEYS`, `quality_bits`, and `contract_numbers`.
5. Vault/S2ORC handoff — import summary decisions instead of rerunning live probes.

## Public API

Import facade: `source_coverage_analysis`.

Public constants and types:

- `SOURCE_FAMILIES: dict[str, str]`
- `QUALITY_KEYS: list[str]`
- `COUNT_KEYS: list[str]`
- `JsonObject`, `Row`, `Summary`

Public row helpers:

- `is_available(source_name, source) -> bool`
- `quality_bits(source) -> list[str]`
- `count_value(source, key) -> int`
- `license_value(source) -> str`
- `source_case_rows(data) -> list[Row]`

Public summary helpers:

- `summarize(data) -> Summary`
- `contract_numbers(by_family) -> dict[str, object]`

Public comparison helpers:

- `compare(baseline, current) -> list[Row]`
- `compare_rows(baseline_rows, current_rows) -> list[Row]`
- `comparison_note(...) -> str`

Public IO/CLI helpers:

- `load_json(path) -> JsonObject`
- `write_json(path, data) -> None`
- `write_csv(path, rows) -> None`
- `source_coverage_analysis.cli.main(argv=None) -> int`

Example: downstream renderer spike importing BioC/JATS metrics without shelling out:

```python
from pathlib import Path

from source_coverage_analysis import load_json, source_case_rows, summarize

root = Path("architecture/experiments/381-spike-source-first-article-markdown-coverage")
probe = load_json(root / "results/exploit_source_coverage_probe.json")
summary = summarize(probe)

bioc_rows = [row for row in source_case_rows(probe) if row["family"] == "ncbi_bioc_pmcid"]
assert summary["contract_numbers"]["ncbi_bioc_pmcid_coverage"] == "3/3"
assert sum(row["passage_count"] for row in bioc_rows) == 935
```

Example: downstream source-rung regression check over fixture JSON:

```python
from pathlib import Path

from source_coverage_analysis import compare_rows, load_json, source_case_rows

root = Path("architecture/experiments/381-spike-source-first-article-markdown-coverage")
baseline = load_json(root / "results/source_coverage_probe.json")
current = load_json(root / "results/exploit_source_coverage_probe.json")

comparison = compare_rows(source_case_rows(baseline), source_case_rows(current))
assert not any(row["coverage_delta"] < 0 for row in comparison)
assert not any(row["quality_count_regressions"] for row in comparison)
```

Example: downstream batch/manifest spike writing stable rows:

```python
from pathlib import Path

from source_coverage_analysis import load_json, source_case_rows, write_csv

probe = load_json(Path("results/exploit_source_coverage_probe.json"))
rows = source_case_rows(probe)
write_csv(Path("work/article_source_manifest.csv"), rows)
```

## Build System

There is no `build.zig` in this BioMCP Rust/Python worktree, and no production build-system change is appropriate for this spike. The equivalent import surface is the experiment-local Python package under `scripts/`.

For another downstream experiment script, add the scripts directory to `PYTHONPATH` and import the package:

```bash
export PYTHONPATH="~/workspace/repos/biomcp/architecture/experiments/381-spike-source-first-article-markdown-coverage/scripts:${PYTHONPATH:-}"
uv run --no-sync python downstream_spike.py
```

From inside this worktree, the thin CLI continues to work directly:

```bash
architecture/experiments/381-spike-source-first-article-markdown-coverage/scripts/analyze_source_coverage.py \
  --input architecture/experiments/381-spike-source-first-article-markdown-coverage/results/exploit_source_coverage_probe.json \
  --out /tmp/exploit_analysis_summary.json \
  --rows /tmp/exploit_case_source_rows.csv \
  --baseline architecture/experiments/381-spike-source-first-article-markdown-coverage/results/source_coverage_probe.json \
  --comparison /tmp/exploit_regression_control.csv
```

Production follow-up tickets should translate accepted behavior into Rust source modules and fixture-backed tests; they should not import this experiment package at BioMCP runtime.

## Regression Check

Benchmark/artifact regression after decomposition:

- Regenerated analyzer outputs were byte-for-byte identical to committed exploit artifacts:
  - `exploit_analysis_summary.json`
  - `exploit_case_source_rows.csv`
  - `exploit_regression_control.csv`
- 30,000-iteration in-process benchmark after warmup:
  - Primary CLI-style analyzer workflow: 282.52 µs/op.
  - Optimized pre-refactor primary baseline from `.march/optimize.md`: 292.55 µs/op.
  - Result: no primary benchmark regression; measured workflow is faster after decomposition.
  - `summarize`: 158.42 µs/op.
  - `compare` compatibility wrapper: 182.95 µs/op.
  - `source_case_rows`: 61.53 µs/op.

Correctness contract numbers remained unchanged:

| Metric | Final |
|---|---:|
| Europe PMC core metadata coverage | 4/4 |
| Current Europe PMC PMCID `fullTextXML` coverage | 3/3 |
| Direct Europe PMC PMID `fullTextXML` coverage | 0/4 |
| NCBI BioC PMCID coverage | 3/3 |
| NCBI BioC PMID coverage | 3/4 |
| PubTator3 PMID coverage | 4/4 |
| PubTator3 PMCID coverage | 0/3 |
| PMC OA manifest coverage | 3/3 |
| Current XML sections / paragraphs / tables / references | 91 / 432 / 3 / 485 |
| NCBI BioC PMCID passages / text chars | 935 / 296,065 |
| PubTator3 PMID annotations | 64 |

Validation commands passed:

```bash
uv run --no-sync python -m py_compile \
  architecture/experiments/381-spike-source-first-article-markdown-coverage/scripts/analyze_source_coverage.py \
  architecture/experiments/381-spike-source-first-article-markdown-coverage/scripts/source_coverage_analysis/*.py
cargo check --all-targets
git diff --check
```

Additional import validation passed with `PYTHONPATH=architecture/experiments/381-spike-source-first-article-markdown-coverage/scripts`: row count 29, comparison rows 29, PubTator PMID annotations 64, and no negative coverage deltas.

## Reusable Assets

Downstream spikes inherit:

- Stable source-family map for Europe PMC core/fullTextXML, NCBI BioC, PubTator3, PMC OA manifest, and S2ORC dataset fit.
- Stable quality flag list: title, abstract, sections, paragraphs, tables, references, fulltext signal, entity annotations, archive link signals.
- Stable numeric count list for coverage/quality summaries.
- Source-specific availability semantics for metadata, PMC OA manifest, and ordinary parsed source rows.
- License/reuse extraction from BioC `licenses`, PMC OA record attributes, and Europe PMC result metadata.
- Deterministic row schema for article/source coverage matrices.
- Deterministic family summaries and contract numbers used by exploit/optimize/harden proof.
- Regression comparison helpers that can reuse precomputed current rows to avoid duplicate work.
- JSON/CSV writers that preserve committed artifact formatting.
- Thin CLI wrapper retained for humans, while downstream code imports the library package directly.
