# Disease

Use disease commands for normalization and disease-centric cross-entity pivots.
For cancer-outcome questions, the disease `survival` section adds SEER Explorer
survival context without creating a separate survival entity.

## Search diseases

```bash
biomcp search disease -q melanoma --limit 5
biomcp search disease -q glioblastoma --source mondo --limit 5
```

Search resolves common labels toward canonical ontology-backed identifiers.

## Get disease records

By label:

```bash
biomcp get disease melanoma
```

By MONDO identifier:

```bash
biomcp get disease MONDO:0005105
```

The base disease card includes concise OpenTargets gene-score summaries when OpenTargets
returns ranked associated targets. Prefer canonical `MONDO:<id>` values in automation:
they are the stable form BioMCP uses for normalization and fallback repair.
When ranked disease-gene context is present, the `See also:` block also promotes
the strongest follow-up gene pivot before the generic disease-level searches,
for example `biomcp get gene SCN1A clingen constraint` on a Dravet syndrome
gene card.
The default disease card's `More:` block keeps `genes`, `pathways`, and
`phenotypes` visible while also surfacing `survival` and `funding` so those
opt-in sections stay discoverable from the base card.

## Disease sections

Genes (Monarch-backed rows plus additive CIViC and OpenTargets disease-gene associations; OpenTargets scores attach to any rendered row with a matching target score):

```bash
biomcp get disease MONDO:0005105 genes
```

Phenotypes (compact `Key Features` summary plus the comprehensive HPO annotation list):

```bash
biomcp get disease MONDO:0005105 phenotypes
```

When BioMCP can extract a reliable disease summary, the phenotype section renders
`### Key Features` above the HPO table. That summary is also exposed as
`key_features[]` in `--json` output. The table remains the comprehensive phenotype
annotation list, and the existing completeness note still applies.

Clinical features (Monarch/HPO phenotype rows):

```bash
biomcp get disease <name_or_id> clinical_features
```

This opt-in section reuses the Monarch/HPO phenotype backend and exposes those
backend rows directly as clinical features. Unsupported diseases return a
truthful Monarch/HPO empty state rather than fabricated rows, and the section is
not included in `biomcp get disease <name_or_id> all`.

Variants (CIViC disease-associated variants):

```bash
biomcp get disease MONDO:0005105 variants
```

When the variants section is loaded, JSON also exposes `top_variant` as the
highest-ranked CIViC-backed association, and markdown shows the same compact
anchor above the full variants table.

Models (Monarch model-organism evidence):

```bash
biomcp get disease MONDO:0005105 models
```

Pathways (associated pathways):

```bash
biomcp get disease MONDO:0005105 pathways
```

Prevalence (prevalence data):

```bash
biomcp get disease MONDO:0005105 prevalence
```

Survival (SEER Explorer 5-year relative survival by sex for mapped cancers):

```bash
biomcp get disease "chronic myeloid leukemia" survival
```

The survival section is filtered to all ages and all races / ethnicities. JSON
records the request in `section_outcomes.survival`: `data` credits SEER Explorer,
`empty` means the disease did not map to usable SEER evidence, and `unavailable`
reports a source failure without crediting the failed provider. Markdown reports
unavailability in-band; the existing `survival_note` remains for compatibility.

Diagnostic-test pivot (GTR and WHO IVD tests for the condition):

```bash
biomcp get disease tuberculosis diagnostics
```

The disease diagnostic card is capped at 10 rows so it stays terminal-sized.
When rows exist, BioMCP prints a `See also:` command such as
`biomcp search diagnostic --disease tuberculosis --source all --limit 50` for
the broader paged diagnostic search; use `--offset` on `search diagnostic` to
continue paging.

Funding (NIH Reporter grants for the requested disease phrase, with canonical-name fallback for identifier lookups, over the most recent 5 NIH fiscal years):

```bash
biomcp get disease "chronic myeloid leukemia" funding
biomcp get disease "Marfan syndrome" funding
```

The diagnostics, DisGeNET, funding, and clinical features sections stay opt-in
and are not included in `biomcp get disease <name_or_id> all`.

CIViC (clinical evidence):

```bash
biomcp get disease MONDO:0005105 civic
```

Combined sections:

```bash
biomcp get disease MONDO:0005105 genes phenotypes variants models
biomcp get disease "Marfan syndrome" funding
biomcp get disease "chronic myeloid leukemia" survival
biomcp get disease MONDO:0005105 all
```

## Helper commands

```bash
biomcp disease trials melanoma --limit 5
biomcp disease trials "Rett Syndrome" --limit 5
biomcp disease drugs melanoma --limit 5
biomcp disease articles "Lynch syndrome" --limit 5
```

Disease trial pivots send the supplied disease label literally.

## Phenotype-to-disease search

Use HPO term sets for ranked disease candidates:

```bash
biomcp search phenotype "HP:0001250 HP:0001263" --limit 10
```

You can pass terms space-separated or comma-separated.

## Typical disease-centric workflow

1. Normalize disease label.
2. Pull disease sections (`genes`, `phenotypes`, `variants`, `models`, and `survival` for cancers) for context.
3. Use normalized concept for trial or article searches.

Example:

```bash
biomcp get disease MONDO:0005105 genes phenotypes
biomcp get disease "chronic myeloid leukemia" survival
biomcp search trial -c melanoma --status recruiting --limit 5
biomcp search article -d melanoma --limit 5
```

## JSON mode

```bash
biomcp --json get disease MONDO:0005105 all
biomcp --json search phenotype "HP:0001250 HP:0001263"
```

`biomcp --json get disease MONDO:0005105` includes `top_gene_scores[]` with
overall OpenTargets scores and any available GWAS, rare-variant, or somatic subtype scores.

## Practical tips

- Prefer MONDO IDs in automation workflows.
- Keep raw labels in user-facing notes for readability.
- Pair disease normalization with biomarker filters for trial matching.

## Related guides

- [Trial](trial.md)
- [Article](article.md)
- [Data sources](../reference/data-sources.md)
