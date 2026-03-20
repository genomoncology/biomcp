# How to: annotate variants

BioMCP provides lightweight variant annotation suitable for triage and workflow automation.

## Choose an ID format

`biomcp get variant` supports:

- rsID: `rs113488022`
- HGVS genomic: `chr7:g.140453136A>T`
- Gene + protein change: `BRAF V600E`

Examples:

```bash
biomcp get variant rs113488022
biomcp get variant "chr7:g.140453136A>T"
biomcp get variant "BRAF V600E"
```

`get variant` stays exact-only. If you have shorthand like `PTPN22 620W` or
`R620W`, resolve it through `search variant` first.

## Search shorthand aliases

`biomcp search variant` accepts a few common search-only shorthand forms in
addition to the exact identifiers above:

- Gene + residue alias: `PTPN22 620W`
- Gene flag + protein shorthand: `biomcp search variant -g PTPN22 R620W`

Examples:

```bash
biomcp search variant "PTPN22 620W" --limit 10
biomcp search variant -g PTPN22 R620W --limit 10
```

Standalone protein shorthand like `R620W` is still too ambiguous to run
automatically. BioMCP returns variant-specific guidance instead of silently
searching the wrong entity.

## Filter search results

```bash
biomcp search variant -g BRCA1 --significance pathogenic --limit 10
```

Add frequency and score filters:

```bash
biomcp search variant -g BRCA1 --max-frequency 0.01 --min-cadd 20 --limit 10
```

## Optional enrichments

- OncoKB (set `ONCOKB_TOKEN` for the production endpoint)
- cBioPortal mutation summaries (best effort)

If these services are unavailable, BioMCP degrades gracefully and will still return core annotations.
