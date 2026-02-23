# Benchmarks

This page defines practical benchmark patterns for BioMCP.

The objective is reproducible command behavior, not synthetic leaderboard numbers.

## Benchmark goals

- Compare latency across entities.
- Track output-size drift.
- Validate cache behavior.
- Catch regressions in command startup overhead.

## Baseline command set

Use a fixed baseline so runs are comparable over time.

```bash
biomcp get gene BRAF
biomcp get variant "BRAF V600E"
biomcp get trial NCT02576665
biomcp search article -g BRAF --limit 5
```

## Latency measurement

Use repeated runs and report median + spread.

Example with `hyperfine`:

```bash
hyperfine -m 10 'biomcp get gene BRAF' 'biomcp search trial -c melanoma --limit 5'
```

Recommendations:

- run on stable network,
- avoid mixed-background workloads,
- capture raw command outputs for auditing.

## Output-size tracking

Track markdown and JSON sizes independently.

```bash
biomcp get gene BRAF | wc -c
biomcp --json get gene BRAF | wc -c
```

Why this matters:

- markdown size impacts prompt context cost,
- JSON size affects API and queue throughput,
- sudden growth can signal response-shape regressions.

## Cache-behavior checks

Compare first call vs second call for cache-eligible endpoints.

```bash
biomcp get article 22663011
biomcp get article 22663011
```

Capture timing for both runs to verify expected improvement.

## Date-validation contract checks

Invalid dates should fail before network calls.

Examples:

```bash
biomcp search article -g BRAF --since 2024-13-01 --limit 1
biomcp search article -g BRAF --since 2024-02-30 --limit 1
```

Expected behavior:

- immediate validation error,
- no long wait for upstream API timeouts.

## Cache-directory contract checks

Downloaded artifacts should use platform cache paths.

Use command help and health output to validate paths in your environment.

## Reporting template

When sharing benchmark results, include:

- command list,
- machine details,
- network assumptions,
- cache warm/cold status,
- median and p95 latency,
- output byte counts.

## Caveats

- Public APIs can throttle unpredictably.
- Upstream schema changes can affect command duration.
- Geographic distance to API hosts influences baseline latency.

## Recommended cadence

- Run quick checks on every release candidate.
- Run fuller benchmark suites before major parser or rendering changes.
