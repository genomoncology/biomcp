# Variant

Use variant commands for compact annotation, source-backed interpretation context,
and optional predictive and population-genetics sections.

## Accepted variant identifiers

BioMCP supports multiple input forms:

- rsID: `rs113488022`
- HGVS genomic: `chr7:g.140453136A>T`
- gene-protein form: `BRAF V600E`

## Get a variant record

```bash
biomcp get variant rs113488022
biomcp get variant "chr7:g.140453136A>T"
biomcp get variant "BRAF V600E"
```

The default output favors concise, clinically relevant context first.

## Request variant sections

Prediction section:

```bash
biomcp get variant "BRAF V600E" predict
```

ClinVar-focused section:

```bash
biomcp get variant rs113488022 clinvar
```

Population section:

```bash
biomcp get variant "chr7:g.140453136A>T" population
```

CIViC section:

```bash
biomcp get variant "BRAF V600E" civic
```

GWAS section (trait associations from GWAS Catalog):

```bash
biomcp get variant rs7903146 gwas
```

All supported sections:

```bash
biomcp get variant rs113488022 all
```

## Search variants

By gene and protein change:

```bash
biomcp search variant -g BRAF --hgvsp V600E --limit 5
```

By significance:

```bash
biomcp search variant -g BRCA1 --significance pathogenic --limit 5
```

With population and score filters:

```bash
biomcp search variant -g BRCA1 --max-frequency 0.01 --min-cadd 20 --limit 5
```

## Search GWAS associations

By gene:

```bash
biomcp search gwas -g TCF7L2 --limit 10
```

By trait:

```bash
biomcp search gwas --trait "type 2 diabetes" --limit 10
```

Trait search uses GWAS Catalog trait endpoints first, then study-association fallback paths when needed.

## Optional enrichment

Variant base output may include cBioPortal enrichment when available.
OncoKB is accessed explicitly via `biomcp variant oncokb "<gene> <variant>"` and requires `ONCOKB_TOKEN`.

## Prediction requirements

Prediction sections may require `ALPHAGENOME_API_KEY` depending on source path.
Unsupported inputs are surfaced with explicit validation messages.

## JSON mode

```bash
biomcp --json get variant "BRAF V600E"
biomcp --json get variant rs7903146 gwas
biomcp --json search gwas --trait "type 2 diabetes"
```

## Related guides

- [How to annotate variants](../how-to/annotate-variants.md)
- [How to predict effects](../how-to/predict-effects.md)
- [Gene](gene.md)
- [Trial](trial.md)
