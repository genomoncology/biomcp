# Optimize: Variant to protein-structure annotation

## Starting Baseline

Starting contract from `.march/exploit.md`:

| Variant | Residue | PDB count | AlphaFold ID | Overlapping InterPro domains | Direct observed latency |
|---|---:|---:|---|---:|---:|
| BRAF V600E | 600 | 131 | P15056 | 4 | 32003 ms |
| TP53 R175H | 175 | 295 | P04637 | 4 | 7022 ms |
| ROS1 G2032R | 2032 | 5 | P08922 | 5 | 27006 ms |
| **Sum** |  |  |  |  | **66031 ms** |

Fresh reproducible optimization baseline after one transient InterPro 503 retry:

| Variant | MyVariant | UniProt | InterPro | Cancerhotspots via CLI | Total latency |
|---|---:|---:|---:|---:|---:|
| BRAF V600E | 304 ms | 538 ms | 14547 ms | 2479 ms | 17868 ms |
| TP53 R175H | 295 ms | 715 ms | 12126 ms | 3170 ms | 16306 ms |
| ROS1 G2032R | 295 ms | 435 ms | 16458 ms | 2083 ms | 19271 ms |
| **Sum** |  |  |  |  | **53445 ms** |

Correctness at baseline matched the exploit contract: requested residues, PDB counts, AlphaFold IDs, InterPro overlap counts, and source-labelled Cancerhotspots presence all reproduced.

## Optimization Passes

### Pass 1 — direct Cancerhotspots probe

- Hotspot: `cancerhotspots_probe` in `architecture/experiments/variant-structure-annotation/scripts/run_experiments.py`.
- Cost before: 2479 ms / 3170 ms / 2083 ms, because the experiment spawned `biomcp --json --no-cache get variant ... all` to obtain one recurrence field.
- Approach: replace the broad CLI subprocess with a direct Cancerhotspots by-gene HTTP request and a small in-process residue/alternate-amino-acid filter.
- Result: committed (`b5bd0019`).

| Variant | Before total | After total | Cancerhotspots before → after |
|---|---:|---:|---:|
| BRAF V600E | 17868 ms | 12924 ms | 2479 → 165 ms |
| TP53 R175H | 16306 ms | 18535 ms | 3170 → 199 ms |
| ROS1 G2032R | 19271 ms | 18769 ms | 2083 → 134 ms |
| **Sum** | **53445 ms** | **50228 ms** | **7732 → 498 ms** |

The targeted component improved 93.6%; summed direct latency improved 6.0%. TP53's total was worse because InterPro live latency moved independently.

### Pass 2 — smaller InterPro page

- Hotspot: `interpro_domains` InterPro request.
- Approach: reduce InterPro `page_size` from 25 to 10.
- Result: reverted.

| Variant | Before overlap count | After overlap count | Before total | After total |
|---|---:|---:|---:|---:|
| BRAF V600E | 4 | 3 | 12924 ms | 22366 ms |
| TP53 R175H | 4 | 4 | 18535 ms | 10033 ms |
| ROS1 G2032R | 5 | 3 | 18769 ms | 21917 ms |

The smaller page dropped valid BRAF and ROS1 overlapping annotations, so it failed correctness.

### Pass 3 — overlap independent source I/O

- Hotspot: direct-source orchestration in `run_direct_join`.
- Approach: run MyVariant, UniProt, and Cancerhotspots concurrently; start InterPro as soon as MyVariant identifies the requested residue; record wall-clock latency for the joined result.
- Result: committed (`4242946c`).

| Variant | Before direct latency after Pass 1 | After wall latency | Correctness |
|---|---:|---:|---|
| BRAF V600E | 12924 ms | 15636 ms | preserved |
| TP53 R175H | 18535 ms | 14376 ms | preserved |
| ROS1 G2032R | 18769 ms | 7291 ms | preserved |
| **Sum** | **50228 ms** | **37303 ms** | preserved |

Summed wall latency improved 25.7% versus Pass 1. BRAF was individually slower in that run because InterPro took 15345 ms, but the aggregate benchmark moved strongly and correctness stayed intact.

### Pass 4 — reuse a requests session

- Hotspot: repeated HTTP connection setup in `http_get`.
- Approach: replace per-call `requests.get(...)` with a module-level `requests.Session` carrying the common headers.
- Result: reverted.

| Variant | Before wall latency | After wall latency | Correctness |
|---|---:|---:|---|
| BRAF V600E | 15636 ms | 19254 ms | preserved |
| TP53 R175H | 14376 ms | 3901 ms | preserved |
| ROS1 G2032R | 7291 ms | 14874 ms | preserved |
| **Sum** | **37303 ms** | **38029 ms** | preserved |

Correctness was preserved, but summed latency regressed by 1.9%. Provider jitter dominated any connection-reuse benefit.

## Final Numbers

Final validation commands:

```bash
architecture/experiments/variant-structure-annotation/scripts/run_experiments.py --approach direct
make lint
make test
make spec
```

Latest post-optimization direct run:

| Variant | Residue | PDB count | AlphaFold ID | InterPro overlaps | Cancerhotspots | Final latency |
|---|---:|---:|---|---:|---|---:|
| BRAF V600E | 600 | 131 | P15056 | 4 | present | 23954 ms |
| TP53 R175H | 175 | 295 | P04637 | 4 | present | 10944 ms |
| ROS1 G2032R | 2032 | 5 | P08922 | 5 | present | 6981 ms |
| **Sum** |  |  |  |  |  | **41879 ms** |

Best clean post-optimization run after committed optimizations was 37303 ms summed wall latency. The latest run is reported as final because it is the last validation run; both preserve correctness and beat the explore summed latency baseline despite per-variant live-source jitter.

Validation status:

- `make lint` passed after removing generated result payloads from git tracking.
- `make test` passed.
- `make spec` passed.

## Total Improvement

| Metric | Starting optimization baseline | Final latest run | Change |
|---|---:|---:|---:|
| BRAF direct latency | 17868 ms | 23954 ms | -34.1% regression from InterPro live noise |
| TP53 direct latency | 16306 ms | 10944 ms | 32.9% faster |
| ROS1 direct latency | 19271 ms | 6981 ms | 63.8% faster |
| Summed direct latency | 53445 ms | 41879 ms | 21.6% faster |
| Summed exploit-contract latency | 66031 ms | 41879 ms | 36.6% faster |
| Cancerhotspots component sum | 7732 ms | about 400-500 ms/run | about 94% faster |
| Correct residue/PDB/AlphaFold/domain/Cancerhotspots contract | pass | pass | unchanged |

Regression control against explore baseline: explore summed latency was 53036 ms (15568 + 17513 + 19955). The final latest summed latency was 41879 ms, 21.0% faster, with the same correctness counts. BRAF's individual final latency was slower than explore because InterPro was noisy, but the aggregate benchmark and all correctness metrics matched or beat the explore control.

## Convergence

Stopped after Pass 4. The latest attempted pass produced less than 5% improvement on the primary metric; it regressed by 1.9% and was reverted. The remaining hotspot is live InterPro endpoint response time. Local overhead from Cancerhotspots probing and source orchestration has been reduced; further gains now require an architectural change rather than another small local optimization.

## Remaining Opportunities

- Add a production cache for InterPro domain ranges and UniProt structure rows. This is the biggest practical latency lever, but it changes runtime/cache behavior and belongs in the follow-on build ticket.
- Parse and store InterPro ranges in BioMCP's normal source model so the CLI does not need an experiment-local join script.
- Consider an operator/live lane with repeated-run medians for latency reporting. Single live runs are too noisy to prove small improvements.
- Keep RCSB residue coordinate probing out of the first contract; it would add another live dependency and more latency.
- Generated experiment result JSON should remain untracked local artifacts; durable writeups/scripts belong under `architecture/experiments/variant-structure-annotation/`.
