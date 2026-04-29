# Study Queries

The `study` surface is BioMCP's local cBioPortal analytics layer, separate from
the remote trial registry surface. These canaries keep the local catalog,
typed query grammar, validation messages, and chartable summaries visible
without pinning install-specific row totals.

## Local Study Discovery

Listing studies should still look like a local dataset catalog, with stable
identity and availability columns that tell operators what data is actually on
disk.

```bash
out="$(../../tools/biomcp-ci study list)"
echo "$out" | mustmatch like "# Study Datasets"
echo "$out" | mustmatch like "| Study ID | Name | Cancer Type | Samples | Available Data |"
echo "$out" | mustmatch '/\| [^|]+ \| [^|]+ \| [^|]+ \| [0-9]+ \| [^|]+ \|/'
```

## Gene-Frequency Summary

Per-study mutation queries should keep a human-readable summary heading and the
variant-class breakout that explains what was counted.

```bash
help_out="$(../../tools/biomcp-ci study query --help)"
out="$(../../tools/biomcp-ci study query --study msk_impact_2017 --gene TP53 --type mutations)"
echo "$help_out" | mustmatch like "Canonical values: mutations, cna, expression."
echo "$help_out" | mustmatch like "Accepted aliases: mutation, copy_number, copy-number, expr"
echo "$out" | mustmatch like "# Study Mutation Frequency: TP53 (msk_impact_2017)"
echo "$out" | mustmatch like "## Top Variant Classes"
echo "$out" | mustmatch like "## Top Protein Changes"
```

## Remote Download Stall Policy

Remote DataHub archives can be large, but a server that starts an archive
response and then stops sending bytes should fail clearly instead of hanging
forever.

```bash
out="$(cargo test download_study_stalled_archive_body_times_out_with_clear_error --lib 2>&1 || true)"
echo "$out" | mustmatch like "download_study_stalled_archive_body_times_out_with_clear_error"
echo "$out" | mustmatch like "test result: ok"
```

## Filter Validation

Filter workflows should reject missing criteria explicitly instead of silently
returning the full cohort.

```bash
out="$(../../tools/biomcp-ci study filter --study brca_tcga_pan_can_atlas_2018 2>&1 || true)"
echo "$out" | mustmatch like "At least one filter criterion is required."
echo "$out" | mustmatch like "--mutated, --amplified, --deleted"
```

## Survival Validation

Survival analysis should stay typed: unknown endpoint names must fail fast and
tell the operator which endpoint vocabulary is valid.

```bash
cohort_out="$(../../tools/biomcp-ci study cohort --study msk_impact_2017 --gene TP53)"
out="$(../../tools/biomcp-ci study survival --study msk_impact_2017 --gene TP53 --endpoint foo 2>&1 || true)"
echo "$cohort_out" | mustmatch like "# Study Cohort: TP53 (msk_impact_2017)"
echo "$cohort_out" | mustmatch like "| Group | Samples | Patients |"
echo "$out" | mustmatch like "Unknown survival endpoint 'foo'."
echo "$out" | mustmatch like "Expected: os, dfs, pfs, dss."
```

## Typed Comparison Validation

Comparison and co-occurrence analytics should reject malformed inputs before
running local cohort work.

```bash
compare_out="$(../../tools/biomcp-ci study compare --study msk_impact_2017 --gene TP53 --type foo --target ERBB2 2>&1 || true)"
co_out="$(../../tools/biomcp-ci study co-occurrence --study msk_impact_2017 --genes TP53 2>&1 || true)"
echo "$compare_out" | mustmatch like "Unknown comparison type 'foo'. Expected: expression, mutations."
echo "$co_out" | mustmatch like "--genes must contain 2 to 10 comma-separated symbols"
```

## Comparison & Chart Output

Study analytics should remain usable from the terminal: comparison summaries
stay tabular, and chart mode still exposes a visible title and axis label.

```bash
cmp_out="$(../../tools/biomcp-ci study compare --study msk_impact_2017 --gene TP53 --type mutations --target ERBB2)"
chart_out="$(../../tools/biomcp-ci study query --study msk_impact_2017 --gene TP53 --type mutations --chart bar)"
echo "$cmp_out" | mustmatch like "# Study Group Comparison: Mutation Rate"
echo "$cmp_out" | mustmatch like "| Group | N | Mutated | Mutation Rate |"
echo "$chart_out" | mustmatch like "TP53 mutation classes"
echo "$chart_out" | mustmatch like "Variant class"
```
