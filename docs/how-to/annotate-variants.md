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
