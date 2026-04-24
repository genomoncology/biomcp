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
echo "$out" | mustmatch like "msk_impact_2017"
```

## Gene-Frequency Summary

Per-study mutation queries should keep a human-readable summary heading and the
variant-class breakout that explains what was counted.

```bash
out="$(../../tools/biomcp-ci study query --study msk_impact_2017 --gene TP53 --type mutations)"
echo "$out" | mustmatch like "# Study Mutation Frequency: TP53 (msk_impact_2017)"
echo "$out" | mustmatch like "## Top Variant Classes"
echo "$out" | mustmatch like "## Top Protein Changes"
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
out="$(../../tools/biomcp-ci study survival --study msk_impact_2017 --gene TP53 --endpoint foo 2>&1 || true)"
echo "$out" | mustmatch like "Unknown survival endpoint 'foo'."
echo "$out" | mustmatch like "Expected: os, dfs, pfs, dss."
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
