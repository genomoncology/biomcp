# Harden: Variant to protein-structure annotation

## Decomposition

The optimized implementation was decomposed from one experiment CLI script into an importable library plus a thin wrapper.

Extracted library code lives under:

- `architecture/experiments/variant-structure-annotation/variant_structure_annotation/types.py`
  - shared input/result types
- `architecture/experiments/variant-structure-annotation/variant_structure_annotation/sources.py`
  - source readers and parsing helpers for MyVariant, UniProt, InterPro, Cancerhotspots, and optional RCSB probing
- `architecture/experiments/variant-structure-annotation/variant_structure_annotation/pipeline.py`
  - optimized orchestration, benchmark workloads, summary projection, and result writing
- `architecture/experiments/variant-structure-annotation/variant_structure_annotation/__init__.py`
  - stable import facade for downstream spikes

The CLI wrapper is now only `architecture/experiments/variant-structure-annotation/scripts/run_experiments.py`. It is 39 lines and only handles argument parsing, selecting the run mode, calling the library, and printing/writing summaries.

This repository is BioMCP, not a Zig project. There is no `build.zig` to update. The equivalent import packaging is `architecture/experiments/variant-structure-annotation/pyproject.toml`, which declares the experiment package and its `requests` dependency.

## Public API

Downstream spikes should import the library and call it directly. They should not shell out to `scripts/run_experiments.py` and should not copy-paste the source join code.

### Import facade

```python
from variant_structure_annotation import (
    DEFAULT_VARIANTS,
    VariantSpec,
    cancerhotspots_probe,
    interpro_domains,
    myvariant_hit,
    run_direct_join,
    run_existing_cli,
    summarize,
    uniprot_record,
    uniprot_summary,
    write_result,
)
```

### Shared types

- `VariantSpec(gene: str, change: str, label: str, accession: str)`
  - the shared input shape for variant/protein joins
- `TimedResult(label: str, ok: bool, latency_ms: int, value: Any | None = None, error: str | None = None)`
  - the benchmark result envelope; includes `to_dict()` for JSON-compatible output

### Source functions

- `normalize_change(change: str) -> str`
- `parse_hgvsp_position(hgvsp: str | None) -> int | None`
- `requested_position_from_hgvsp(hgvsp_values: list[str], requested_change: str) -> int | None`
- `myvariant_hit(gene: str, change: str) -> dict`
- `uniprot_record(accession: str) -> dict`
- `uniprot_summary(record: dict) -> dict`
- `interpro_domains(accession: str, residue: int | None) -> dict`
- `cancerhotspots_probe(gene: str, change: str) -> dict`

### Pipeline functions

- `run_direct_join(variants: Iterable[VariantSpec] | None = None, with_rcsb: bool = False) -> dict`
  - optimized library-first path; runs MyVariant, UniProt, and Cancerhotspots concurrently and starts InterPro after MyVariant identifies the requested residue
- `run_existing_cli(variants: Iterable[VariantSpec] | None = None, biomcp_bin: Path | str | None = None) -> dict`
  - baseline comparison only; this is not the downstream integration path
- `summarize(result: dict) -> dict`
  - stable projection used by regression checks
- `write_result(result: dict, out_dir: Path = OUT_DIR) -> dict`
  - writes detailed and summary JSON results and returns the summary

### Downstream usage examples

Run the optimized join for one variant:

```python
from variant_structure_annotation import VariantSpec, run_direct_join, summarize

result = run_direct_join([
    VariantSpec(
        gene="BRAF",
        change="V600E",
        label="BRAF V600E",
        accession="P15056",
    )
])
summary = summarize(result)
print(summary["variants"][0]["overlap_count"])
```

Call a single reusable source reader:

```python
from variant_structure_annotation import interpro_domains

domains = interpro_domains("P15056", residue=600)
for domain in domains["overlaps"]:
    print(domain["accession"], domain["name"], domain["locations"])
```

Use the default ticket workload and persist results:

```python
from variant_structure_annotation import run_direct_join, write_result

summary = write_result(run_direct_join())
assert summary["n"] == 3
```

## Build System

This spike is Python experiment code inside a Rust BioMCP worktree, so the build-system answer is Python packaging rather than `build.zig`.

The experiment now has:

```toml
[project]
name = "variant-structure-annotation"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = ["requests"]

[tool.setuptools]
packages = ["variant_structure_annotation"]
```

Downstream options:

1. For another experiment beside this one, add the experiment directory to `PYTHONPATH`:

   ```bash
   PYTHONPATH=architecture/experiments/variant-structure-annotation \
     uv run --with requests path/to/downstream_spike.py
   ```

2. For an editable local dependency from a downstream Python harness:

   ```bash
   uv pip install -e architecture/experiments/variant-structure-annotation
   ```

3. For a one-off script, insert the experiment directory into `sys.path` before importing, matching the thin CLI wrapper:

   ```python
   from pathlib import Path
   import sys

   sys.path.insert(0, str(Path("architecture/experiments/variant-structure-annotation").resolve()))
   from variant_structure_annotation import run_direct_join
   ```

## Regression Check

Benchmark command:

```bash
architecture/experiments/variant-structure-annotation/scripts/run_experiments.py --approach all
```

Results after decomposition:

| Approach | BRAF | TP53 | ROS1 | Correctness |
|---|---:|---:|---:|---|
| existing CLI composition | 4530 ms | 5087 ms | 3949 ms | variant/protein structures/protein domains all OK; domain locations still not exposed, as before |
| direct source join | 702 ms | 727 ms | 834 ms | zero regression |
| direct source join + structure links | 1404 ms | 1387 ms | 1269 ms | zero regression |

Direct-source correctness stayed exactly on the optimized contract:

| Variant | Residue | PDB count | AlphaFold ID | InterPro overlaps | Cancerhotspots |
|---|---:|---:|---|---:|---|
| BRAF V600E | 600 | 131 | P15056 | 4 | present |
| TP53 R175H | 175 | 295 | P04637 | 4 | present |
| ROS1 G2032R | 2032 | 5 | P08922 | 5 | present |

Validation commands:

```bash
make lint
make test
make spec
```

Validation results:

- `make lint`: passed
- `make test`: passed, including 2393 Rust tests and 292 Python/docs tests
- `make spec`: passed on rerun, with 72 specs and 28 parallel-isolation tests

Note: the first `make spec` attempt failed in an existing Streamable HTTP host-header spec because its local test server was not reachable on the assigned localhost ports. The immediate rerun passed without code changes, so this was treated as a transient harness/server readiness issue, not a refactor regression.

## Reusable Assets

Downstream spikes inherit:

- `VariantSpec`: shared input shape for gene/protein-change/accession joins
- protein-change parsing helpers: `normalize_change`, `parse_hgvsp_position`, `requested_position_from_hgvsp`
- direct source readers:
  - MyVariant residue/protein-position reader
  - UniProt PDB/AlphaFold cross-reference reader
  - InterPro domain-range reader with residue-overlap filtering
  - Cancerhotspots recurrence reader that avoids shelling out to the BioMCP CLI
- optimized concurrent orchestration in `run_direct_join`
- stable summary projection in `summarize`
- thin CLI wrapper pattern for experiment runners
- experiment-local Python package metadata for direct imports
