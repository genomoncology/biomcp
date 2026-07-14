# Structural-Variant Annotation Layer — Optimization

## Starting Baseline

The ticket sets no separate performance target. The primary metric is full-scale end-to-end CLI throughput; latency is its inverse. Correctness, deterministic output, regression-control performance, and peak RSS are controls.

The exploit contract measured 843.16 documents/s and 1.1860 ms/document over 60 papers. A fresh unchanged-code reproduction established the optimization baseline:

| Corpus | Median | p95 | Throughput | Latency | Peak child RSS | Output SHA-256 |
|---|---:|---:|---:|---:|---:|---|
| Full scale (60 docs) | 0.071988 s | 0.073164 s | 833.47 docs/s | 1.1998 ms/doc | 20,464 KiB | `c12760adda62c54bb684db6f40a542edeacea6b2538ad8db5578afba49225a94` |
| Regression control (16 docs) | 0.038786 s | 0.039948 s | 412.52 docs/s | 2.4241 ms/doc | 20,208 KiB | `495c5fca2c7c37b411176d0d2bede256667dd48c514dcfc1b7ebfc552c2bcd00` |

The reproduction was within 1.2% of the exploit contract. Correctness was 91 TP / 0 FP / 0 FN full scale and 88/0/0 on the corrected regression control.

## Optimization Passes

### Pass 1 — Precompile regex objects

- **Hotspot:** `detect` at `scripts/structural_events.py:116-129`, 0.048 s of a 0.060 s profiled run (80%). Regex dispatch/cache lookup was about 0.002 s; initial compilation was about 0.007 s.
- **Approach:** Compile each regex at module load and call bound `Pattern.finditer`.
- **Before → after:** 833.47 → 837.07 docs/s (+0.43%); 1.1998 → 1.1946 ms/doc; control 412.52 → 402.33 docs/s (-2.47%); RSS 20,464 → 20,548 KiB.
- **Decision:** Reverted. The primary movement was below run noise and the control regressed. Correctness and checksums were unchanged.

### Pass 2 — Group scans by event type

- **Hotspot:** The parser performs 33 whole-document regex scans per document; `detect` remained 0.043 s self / 0.045 s cumulative.
- **Approach:** Combine expressions by event type into eight precompiled alternations, reducing scan count by 76%.
- **Before → after:** Profiled `detect` fell from 0.048 s to 0.039 s (-18.8%), but corrected-control correctness fell from 88/0/0 to 87/0/1.
- **Decision:** Reverted before the full benchmark. Alternation suppressed an overlapping repeated occurrence, violating exact occurrence semantics.

### Pass 3 — Reuse locus parsing

- **Hotspot:** `annotate` called `_loci` twice per event (182 calls); `_loci` cost about 0.001 s in the baseline profile.
- **Approach:** Compute loci once per event and reuse the result.
- **Before → after:** 833.47 → 835.46 docs/s (+0.24%); 1.1998 → 1.1969 ms/doc; p95 0.073164 → 0.076196 s; control 412.52 → 408.91 docs/s (-0.88%); RSS 20,464 → 20,408 KiB.
- **Decision:** Reverted. The primary movement was noise, p95 worsened, and the theoretical end-to-end ceiling was under 2%. Correctness and checksums were unchanged.

## Final Numbers

All optimization changes were reverted. A final seven-run measurement on the restored implementation produced:

| Corpus | Median | p95 | Throughput | Latency | Peak child RSS | Output SHA-256 |
|---|---:|---:|---:|---:|---:|---|
| Full scale (60 docs) | 0.072069 s | 0.073917 s | 832.54 docs/s | 1.2011 ms/doc | 20,852 KiB | `c12760adda62c54bb684db6f40a542edeacea6b2538ad8db5578afba49225a94` |
| Regression control (16 docs) | 0.039601 s | 0.040740 s | 404.04 docs/s | 2.4750 ms/doc | 20,596 KiB | `495c5fca2c7c37b411176d0d2bede256667dd48c514dcfc1b7ebfc552c2bcd00` |

Correctness remains 91/0/0 full scale and 88/0/0 on the corrected regression control. `make lint`, `make test`, and `make spec` pass.

## Total Improvement

Because no valid pass improved the primary metric, the implementation is unchanged. Baseline-to-final differences are measurement noise, not a code improvement.

| Metric | Reproduced baseline | Final | Change |
|---|---:|---:|---:|
| Full-scale throughput | 833.47 docs/s | 832.54 docs/s | -0.11% |
| Full-scale latency | 1.1998 ms/doc | 1.2011 ms/doc | +0.11% |
| Full-scale median | 0.071988 s | 0.072069 s | +0.11% |
| Full-scale p95 | 0.073164 s | 0.073917 s | +1.03% |
| Full-scale peak RSS | 20,464 KiB | 20,852 KiB | +1.90% |
| Regression throughput | 412.52 docs/s | 404.04 docs/s | -2.06% |
| Regression latency | 2.4241 ms/doc | 2.4750 ms/doc | +2.10% |
| Full-scale correctness | 91/0/0 | 91/0/0 | unchanged |
| Regression correctness | 88/0/0 | 88/0/0 | unchanged |
| Output checksums | contract values | contract values | unchanged |

The final regression throughput still beats the exploit contract's 402.88 docs/s, and regression correctness matches the perfect explore baseline.

## Convergence

Optimization stopped after three consecutive passes failed to deliver a valid primary-metric improvement, satisfying the convergence rule. The only material profile improvement (-18.8% in `detect`) compromised repeated-occurrence correctness. Both semantics-preserving passes were below the 5% threshold and ordinary run-to-run noise.

The one-shot Python CLI is near the useful minimum for this architecture: interpreter/import/regex compilation overhead and 33 correctness-preserving scans dominate a workload of only 60 short documents.

## Remaining Opportunities

- A single scanner that explicitly supports overlapping matches could reduce whole-text scans without losing repeated occurrences, but requires an architectural parser change and broader semantic contracts.
- A persistent process or batch service would amortize interpreter and regex-compilation startup; it would change the measured deployment shape.
- A Rust implementation or specialized multi-pattern engine could improve scanning substantially, but the ticket forbids promoting experimental regexes into production and the blind-quality bar has not been met.
- SIMD or a literal-prefix dispatch automaton may help at much larger corpus sizes; this 60-document one-shot benchmark is too small to justify the complexity.
