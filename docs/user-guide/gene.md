# Gene

Use gene commands to retrieve canonical metadata and targeted biological context.

## What the gene guide covers

- symbol-based retrieval,
- lightweight search,
- section expansion,
- JSON output for downstream systems.

## Search genes

Start with search when you are unsure of symbol spelling or want to inspect alias candidates.
Known aliases that map to one canonical human gene can also be passed directly to `get gene`.

```bash
biomcp search gene BRAF --limit 5
```

Useful fields in search output typically include symbol, Entrez ID, and species.

## Get a gene record

```bash
biomcp get gene BRAF
```

The default gene view is concise and intended for orientation. When MyGene.info
returns genomic coordinates, BioMCP labels them with the GRCh38 genome build
(`Chromosome (GRCh38): ...`) so coordinate consumers do not have to infer the
reference. Its `More:` block keeps `pathways`, `ontology`, and `diseases`
visible and now also surfaces `funding` as a direct follow-up from the base
card.

## Request deeper sections

BioMCP expands detail via positional sections.

Pathway view:

```bash
biomcp get gene BRAF pathways
```

Disease associations:

```bash
biomcp get gene BRAF diseases
```

Ontology terms:

```bash
biomcp get gene BRAF ontology
```

Protein summary:

```bash
biomcp get gene BRAF protein
```

When UniProt exposes legacy protein names, the protein section includes an
`Also known as:` line with alternative full names and short names from UniProt.
When UniProt exposes alternative products, the same section also includes an
`Isoforms (N)` line with isoform names and the displayed isoform length when
that length is available from the base UniProt record.

GO terms and interactions:

```bash
biomcp get gene BRAF go interactions
```

CIViC evidence summary:

```bash
biomcp get gene BRAF civic
```

Tissue expression (GTEx):

```bash
biomcp get gene BRAF expression
```

Protein tissue expression and localization (Human Protein Atlas):

```bash
biomcp get gene BRAF hpa
```

Druggability profile (DGIdb interactions plus OpenTargets tractability and safety):

```bash
biomcp get gene BRAF druggability
```

Funding context (NIH Reporter grants mentioning the canonical symbol in the most recent 5 NIH fiscal years):

```bash
biomcp get gene ERBB2 funding
```

Diagnostic-test pivot (GTR tests for the gene):

```bash
biomcp get gene BRCA1 diagnostics
```

The diagnostics and funding sections are opt-in and are not included in
`biomcp get gene <symbol> all`.

Gene-disease validity (ClinGen):

```bash
biomcp get gene BRAF clingen
```

When ClinGen validity rows are present, the gene card's `See also:` block
promotes a recruiting-trial search keyed to the newest reviewed disease label
already shown on the card, ahead of the generic gene pivots.

Constraint metrics (gnomAD):

```bash
biomcp get gene BRAF constraint
```

Multiple sections can be chained:

```bash
biomcp get gene BRAF pathways diseases
```

All supported sections:

```bash
biomcp get gene BRAF all
```

To add funding, request it explicitly:

```bash
biomcp get gene BRAF funding
```

## Helper commands

```bash
biomcp gene trials BRAF --limit 5
biomcp gene trials SHANK3 --limit 5
biomcp gene drugs BRAF --limit 5
biomcp gene pathways BRAF
biomcp gene articles BRAF
biomcp gene definition BRAF
```

For CTGov, supported rare-disease genes such as `SHANK3` reuse the shared
trial plan so the search can include the associated condition expansion. Other
genes and non-CTGov sources keep the literal gene trial search.

## Common workflows

### Clinical trial pivot

```bash
biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5
```

### Literature pivot

```bash
biomcp search article -g BRAF -d melanoma --limit 5
```

### Variant pivot

```bash
biomcp search variant -g BRAF --limit 5
```

## Error handling expectations

If a section name is unsupported, BioMCP returns an explicit unknown-section message
with hints about valid section names.

## JSON mode

Use JSON for pipelines or agent post-processing.

```bash
biomcp --json get gene BRAF
```

`biomcp --json get gene BRAF druggability` includes DGIdb interaction fields plus
OpenTargets `tractability[]` modality summaries and `safety_liabilities[]` event summaries.

## Practical tips

- Keep section requests narrow for better focus.
- Start with one section, then add another only if needed.
- Use `search` first when symbol ambiguity is possible.

## Related guides

- [Variant](variant.md)
- [Article](article.md)
- [Trial](trial.md)
- [Protein](protein.md)
- [Progressive disclosure](../concepts/progressive-disclosure.md)
