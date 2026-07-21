# Variant

Use variant commands for compact annotation, source-backed interpretation context,
and optional predictive and population-genetics sections.

## Accepted variant identifiers

BioMCP supports multiple input forms:

- rsID: `rs113488022`
- HGVS genomic: `chr7:g.140453136A>T`
- gene-protein form: `BRAF V600E`, `BRAF p.Val600Glu`

These exact formats are accepted by `biomcp get variant` and the exact-ID
helper commands.

## Search variants

By gene and protein change:

```bash
biomcp search variant -g BRAF --hgvsp V600E --limit 5
biomcp search variant -g BRAF --hgvsp p.Val600Glu --limit 5
biomcp search variant BRAF p.Val600Glu --limit 5
```

By residue alias shorthand:

```bash
biomcp search variant "PTPN22 620W" --limit 5
```

By protein shorthand when gene context is already supplied:

```bash
biomcp search variant -g PTPN22 R620W --limit 5
```

Standalone protein shorthand like `R620W` returns variant-specific recovery
guidance instead of falling back to gene or condition discovery.

Protein searches using `--hgvsp` or positional `GENE CHANGE`, plus coding-HGVS and rsID searches, are exact identity routes. BioMCP keeps
the supplied spelling, normalizes aliases only for comparison, and excludes
MyVariant candidates whose source gene or allele facts contradict the request.
Exact JSON includes `requested_variant` and a `resolution` status: `resolved`
for one source-proved identity, `ambiguous` when source evidence is incomplete
or multiple identities remain, and `unresolved` when exhaustive candidates are
absent or contradictory. Retained rows expose the provider's complete
`source_identity` arrays and the source-derived `matched_alias` that proved the
match.

Gene-only, residue-alias, significance, consequence, score, population, and
other discovery filters remain broad searches and omit strict identity metadata.

By significance:

```bash
biomcp search variant -g BRCA1 --significance pathogenic --limit 5
```

With population and score filters:

```bash
biomcp search variant -g BRCA1 --max-frequency 0.01 --min-cadd 20 --limit 5
```

Consequence, ClinVar review-status, and field-presence filters can be combined:

```bash
biomcp search variant -g BRAF --consequence missense_variant --limit 5
biomcp search variant -g BRCA1 --review-status 2 --limit 5
biomcp search variant -g BRAF --has revel --limit 5
biomcp search variant -g BRAF --missing revel --limit 5
```

`--has` and `--missing` accept `cadd`, `revel`, `gerp`, `clinvar`, `gnomad`,
`dbsnp`, `snpeff`, `civic`, and `cosmic`. Unsupported consequence,
review-status, or field values exit with a typed `invalid_argument` error rather
than reporting an empty successful search.

CADD and GERP thresholds must be finite. Results filtered with `--gerp-min`
include the qualifying `gerp` score in each row.

## Get a variant record

```bash
biomcp get variant rs113488022
biomcp get variant "chr7:g.140453136A>T"
biomcp get variant "BRAF V600E"
biomcp get variant "BRAF p.Val600Glu"
```

The default output favors concise, clinically relevant context first.
When BioMCP can derive a compact literature-facing alias, variant search rows
and detail cards also show `Legacy Name`, and JSON output includes
`legacy_name`.

Shorthand such as `PTPN22 620W` or `R620W` is not treated as an exact variant
ID. Use `biomcp search variant` for those inputs.

## Normalize transcript HGVS

Use the explicit normalization proxy when you already have a transcript HGVS
string and want source-labelled output from Mutalyzer and VariantValidator:

```bash
biomcp variant normalize all NM_000248.3:c.135del
biomcp variant normalize all 'NM_004448.2:c.829G>T'
biomcp variant normalize mutalyzer NM_000248.3:c.135del
biomcp variant normalize variantvalidator 'NM_004448.2:c.829G>T'
```

JSON output always writes a parseable object on exit 0. It preserves the submitted
`input`, an aggregate `status`, a `results` list of normalized forms, a `message`,
one result per service with each service `status`, source-returned
transcript/normalized/genomic/protein fields, warnings such as VariantValidator
`TranscriptVersionWarning`, and `_meta.next_commands`. When no provider returns a
normalized form, `status` is `no_result` and `results` is an empty list. Markdown
VariantValidator genomic descriptions are labeled as GRCh38 because BioMCP calls
VariantValidator's GRCh38 normalization endpoint.

This helper does not parse messy report prose, does not choose, select, guess,
or infer transcripts, and does not classify variants or provide clinical
interpretation/clinical meaning.

## Request variant sections

The default variant card includes a one-line therapeutic-evidence pointer. When
cached MyVariant CIViC evidence is already in the fetched variant payload, the
line reports the cached predictive-item count and points to `get variant <id>
civic`; otherwise it prints the bare `get variant <id> civic` next-command. The
default card does not run the live CIViC GraphQL section call.

Prediction section:

```bash
biomcp get variant "BRAF V600E" predict
```

ClinVar-focused section:

```bash
biomcp get variant rs113488022 clinvar
```

ClinVar JSON also exposes `top_disease` when condition aggregation is available,
reusing the highest-ranked ClinVar condition row already shown in the section.
The runtime `See also:` block is significance-aware: pathogenic or likely
pathogenic variants keep the gene and drug-target pivots near the top, while
VUS / uncertain-significance variants add a literature search anchored by the
variant alias and any available disease context before the generic drug-target
fallback.

Population section:

```bash
biomcp get variant "chr7:g.140453136A>T" population
```

Population JSON exposes additive compact frequency fields:
`allele_frequency_raw` and `allele_frequency_percent`. Markdown keeps the raw
gnomAD AF line and appends the compact percent inline.

CIViC section:

```bash
biomcp get variant "BRAF V600E" civic
```

The CIViC section includes cached MyVariant rows plus live GraphQL context when
available. It also carries a currency caveat and points to literature and drug
cross-check commands because curated CIViC evidence can lag current standard of
care.

GWAS section (trait associations from GWAS Catalog):

```bash
biomcp get variant rs7903146 gwas
```

GWAS JSON exposes `supporting_pmids` as an ordered, deduplicated array when the
section loads successfully. `null` means either the GWAS section was not loaded
or GWAS Catalog was temporarily unavailable. In the unavailable case, JSON also
includes `gwas_unavailable_reason` with a source-specific message.

Predictions (aggregated prediction scores):

```bash
biomcp get variant "BRAF V600E" predictions
```

MyVariant-backed prediction entries can include REVEL, AlphaMissense, ClinPred,
SIFT, MetaRNN, `BayesDel add-AF`, and `BayesDel no-AF`.
The two BayesDel flavors remain separate source scores; BioMCP does not apply a
clinical threshold or classify pathogenicity from either score.

Conservation (GERP, phyloP):

```bash
biomcp get variant rs113488022 conservation
```

COSMIC (somatic mutation data):

```bash
biomcp get variant "BRAF V600E" cosmic
```

CGI (Cancer Genome Interpreter annotations):

```bash
biomcp get variant "BRAF V600E" cgi
```

cBioPortal (frequency data):

```bash
biomcp get variant "BRAF V600E" cbioportal
```

Cancerhotspots.org recurrence counts are loaded by `all` for exact gene/protein
queries such as `BRAF V600E`. JSON includes `cancerhotspots.source`,
`matched_transcript`, `position_count` (residue-level `tumorCount`), and
`same_aa_count` (the exact alternate amino acid count). If the lookup succeeds
but the residue/change is not a cancerhotspots hotspot, those count/provenance
fields are present as JSON `null` under the source-labelled object; upstream
unavailability omits the object instead of emitting zeros.

All supported sections:

```bash
biomcp get variant rs113488022 all
```

`all` includes broad source-backed annotation sections such as ClinVar,
population, conservation, expanded MyVariant prediction scores, cBioPortal,
Cancerhotspots.org, CIViC, CGI, COSMIC, and GWAS. It does not run the
AlphaGenome `predict` section because that path requires `ALPHAGENOME_API_KEY`;
request `predict` explicitly when you want AlphaGenome output.

## Helper commands

```bash
biomcp variant trials "BRAF V600E"     # search trials mentioning this mutation
biomcp variant articles "BRAF V600E"   # union exact article routes for this variant
biomcp variant structure "BRAF V600E"  # residue/domain/PDB/AlphaFold/hotspot context
biomcp variant oncokb "BRAF V600E"     # OncoKB lookup (requires ONCOKB_TOKEN)
```

`variant structure` is an opt-in, network-backed helper. JSON includes the selected
residue, matched HGVSp aliases, other MyVariant/dbNSFP positions, overlapping
InterPro domain ranges, typed UniProt PDB rows, an AlphaFold URL,
Cancerhotspots recurrence, top-level `lookup_outcomes`, warnings, and
`_meta.next_commands`. `lookup_outcomes.domains` and
`lookup_outcomes.cancerhotspots` distinguish `data`, healthy `empty`, local
`inapplicable`, and temporary `unavailable` results. Cancer Hotspots recurrence
is `null` when that lookup is inapplicable or unavailable; its successful data
and empty object shape is unchanged. This helper does not add structure data to
default `get variant` output.

`variant articles` first performs one strict variant resolution. Its default
`--strategy union` combines compatible PubTator variant annotations, normalized
protein/coding/genomic/rsID aliases across the ordinary article backends, and
validated source-backed PubMed citations. BioMCP merges duplicate papers with
all route/source/alias provenance, ranks the union, and applies `--offset` and
`--limit` once. Use `--strategy annotation` or `--strategy lexical` to diagnose
one exact route. If resolution is ambiguous or unresolved, exact routes stay
empty and the default output labels any discovery result as
`best_effort_free_text` with no exact matched alias.

JSON repeats `requested_variant` on every row and reports `resolution`,
`complete`, `truncated`, full pagination state, and per-route `source_status`.
An incomplete provider route keeps available rows but sets `complete: false`,
`truncated: true`, and `pagination.total: null`.

For several variants, pass a JSON array of 1-10 structured identities from a file
or stdin. Each item can use an rsID, complete genomic HGVS, structured genomic
coordinates, gene plus protein change, or coding change plus gene/transcript:

```bash
biomcp --json variant articles --input variants.json --limit 10
cat variants.json | biomcp --json variant articles --input - --debug-plan
```

The batch response preserves input order under `items`, uses compact article rows,
and supplies parseable article-detail follow-ups in `_meta.next_commands`. At most
two items execute concurrently. Work is bounded to 50 logical calls per valid
item and 50 times the item count for the request. `--debug-plan` is JSON-only and
adds normalized aliases, route/provider facts, ranking inputs, budgets, and stop
state; ordinary single and batch output omit plans. The typed MCP
`variant_articles` tool accepts the same structured item fields directly without
server-local paths.

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
Unsupported command inputs still use explicit validation messages. A valid
variant card whose requested lookup lacks a prerequisite remains successful and
reports that lookup as `inapplicable`.

## JSON mode

```bash
biomcp --json get variant "BRAF V600E"
biomcp --json get variant rs7903146 gwas
biomcp --json search gwas --trait "type 2 diabetes"
```

JSON records requested `predict`, `cancerhotspots`, `civic`, `cbioportal`, and
`gwas` source states in `section_outcomes`; `_meta.section_sources` projects the
same state and successful providers. If genomic coordinates, a gene/protein
change, a gene, or an rsID required by one of these lookups is absent, the result
is `inapplicable`: no provider was contacted or credited, and the message names
the missing prerequisite. A contacted Cancer Hotspots no-match is `empty`, while
a source failure is `unavailable` and receives no provider credit.

## Practical tips

- Use `search variant` first for shorthand or ambiguous inputs.
- Start with the base card, then add source sections such as `clinvar`, `civic`, or `population` only when needed.
- Use `all` when you need a one-shot export for review or downstream comparison.

## Related guides

- [How to annotate variants](../how-to/annotate-variants.md)
- [How to annotate variant structure context](../how-to/annotate-variant-structure.md)
- [How to predict effects](../how-to/predict-effects.md)
- [Gene](gene.md)
- [Trial](trial.md)
