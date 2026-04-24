# Source Labels

This spec verifies that entity detail responses and source-aware adverse-event
search output carry explicit source labels in both markdown and JSON output.
Assertions are structural — they check for stable provenance strings, not
volatile upstream data values.

| Section | Command focus | Why it matters |
|---|---|---|
| Markdown labels | `get <entity>` | Confirms visible source attribution at section boundaries |
| Search labels | `search <entity>` | Confirms source-aware search surfaces expose visible provenance |
| JSON section_sources | `get <entity> --json` | Confirms `_meta.section_sources` with stable key/label/sources fields |

## Markdown Source Labels

Each entity type must name its upstream source at visible section boundaries.

```bash
bin="${BIOMCP_BIN:-biomcp}"
gene_out="$("$bin" get gene CFTR all)"
echo "$gene_out" | mustmatch like "Source: NCBI Gene / MyGene.info"
echo "$gene_out" | mustmatch like "## Summary (NCBI Gene)"

gene_json="$("$bin" get gene CFTR all --json)"
echo "$gene_json" | mustmatch like '"section_sources": ['
echo "$gene_json" | mustmatch like '"key": "summary"'
echo "$gene_json" | mustmatch like '"key": "identity"'
echo "$gene_json" | mustmatch like '"label": "NCBI Gene"'
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
drug_out="$("$bin" get drug tamoxifen targets)"
echo "$drug_out" | mustmatch like "## Targets (ChEMBL / Open Targets)"
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
variant_drug_out="$("$bin" get drug rindopepimut targets)"
echo "$variant_drug_out" | mustmatch like "Variant Targets (CIViC): EGFRvIII"
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
disease_out="$("$bin" get disease "cystic fibrosis")"
echo "$disease_out" | mustmatch like "## Definition (MyDisease.info)"
echo "$disease_out" | mustmatch like "Genes (Open Targets):"
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
trial_out="$("$bin" get trial NCT06668103)"
echo "$trial_out" | mustmatch like "Source: ClinicalTrials.gov"

protein_out="$("$bin" get protein P15056 complexes)"
echo "$protein_out" | mustmatch like "Accession: P15056"
if echo "$protein_out" | grep -F "## Complexes (ComplexPortal)" >/dev/null; then
  echo "$protein_out" | mustmatch like "## Complexes (ComplexPortal)"
else
  echo "$protein_out" | mustmatch not like "## Complexes"
fi

pgx_out="$("$bin" get pgx CYP2D6 recommendations)"
echo "$pgx_out" | mustmatch like "## Recommendations (CPIC)"

ae_out="$("$bin" get adverse-event 10329882)"
echo "$ae_out" | mustmatch like "## Reactions (OpenFDA)"
```

```bash
bin="${BIOMCP_BIN:-$(git rev-parse --show-toplevel)/target/release/biomcp}"
bash fixtures/setup-gtr-spec-fixture.sh "$PWD"
bash fixtures/setup-who-ivd-spec-fixture.sh "$PWD"
. "$PWD/.cache/spec-gtr-env"
. "$PWD/.cache/spec-who-ivd-env"
gtr_diag_out="$("$bin" get diagnostic GTR000000001.1)"
echo "$gtr_diag_out" | mustmatch like "Source: NCBI Genetic Testing Registry"

who_diag_out="$("$bin" get diagnostic 'ITPW02232- TC40' conditions)"
echo "$who_diag_out" | mustmatch like "Source: WHO Prequalified IVD"
echo "$who_diag_out" | mustmatch like "## Conditions"
```

## Search Source Labels

VAERS search output should make the dataset boundary explicit instead of
looking like another OpenFDA FAERS table.

```bash
bin="${BIOMCP_BIN:-$(git rev-parse --show-toplevel)/target/release/biomcp}"
bash fixtures/setup-vaers-spec-fixture.sh "$PWD"
. "$PWD/.cache/spec-vaers-env"
search_out="$("$bin" search adverse-event "MMR vaccine" --source vaers --limit 5)"
echo "$search_out" | mustmatch like "## CDC VAERS Summary"
echo "$search_out" | mustmatch like "Source: CDC VAERS"
```

Diagnostic WHO search output should expose source-aware rows and shell-safe
quoted follow-up commands when the identifier contains spaces.

```bash
bin="${BIOMCP_BIN:-$(git rev-parse --show-toplevel)/target/release/biomcp}"
bash fixtures/setup-who-ivd-spec-fixture.sh "$PWD"
. "$PWD/.cache/spec-who-ivd-env"
search_out="$("$bin" search diagnostic --disease HIV --source who-ivd --limit 5)"
echo "$search_out" | mustmatch like "|Source|Genes|Conditions|"
echo "$search_out" | mustmatch like "WHO Prequalified IVD"
echo "$search_out" | mustmatch like 'Use `biomcp get diagnostic "ITPW02232- TC40"` for details.'
```

## Article Fulltext Source Labels

Resolved article full text should name the actual winning source in both
markdown and JSON output. This fixture server exposes deterministic XML, HTML,
and PDF winners so the source label stays truthful across the ladder instead of
depending on live upstream availability.

```bash
bin="${BIOMCP_BIN:-$(git rev-parse --show-toplevel)/target/release/biomcp}"
bash fixtures/setup-article-fulltext-source-fixture.sh "$PWD"
. "$PWD/.cache/spec-article-fulltext-source-env"
xml_out="$("$bin" get article 22663011 fulltext)"
html_out="$("$bin" get article 22663012 fulltext)"
pdf_out="$("$bin" get article 22663013 fulltext --pdf)"
echo "$xml_out" | mustmatch like "## Full Text (Europe PMC XML)"
echo "$html_out" | mustmatch like "## Full Text (PMC HTML)"
echo "$pdf_out" | mustmatch like "## Full Text (Semantic Scholar PDF)"
```

```bash
bin="${BIOMCP_BIN:-$(git rev-parse --show-toplevel)/target/release/biomcp}"
bash fixtures/setup-article-fulltext-source-fixture.sh "$PWD"
. "$PWD/.cache/spec-article-fulltext-source-env"
xml_json="$("$bin" get article 22663011 fulltext --json)"
html_json="$("$bin" get article 22663012 fulltext --json)"
pdf_json="$("$bin" get article 22663013 fulltext --pdf --json)"
echo "$xml_json" | mustmatch like '{"full_text_source":{"kind":"jats_xml","label":"Europe PMC XML","source":"Europe PMC"}}'
echo "$html_json" | mustmatch like '{"full_text_source":{"kind":"html","label":"PMC HTML","source":"PMC"}}'
echo "$pdf_json" | mustmatch like '{"full_text_source":{"kind":"pdf","label":"Semantic Scholar PDF","source":"Semantic Scholar"}}'
echo "$xml_json" | mustmatch like '{"_meta":{"section_sources":[{"key":"fulltext","label":"Full Text","sources":["Europe PMC"]}]}}'
echo "$html_json" | mustmatch like '{"_meta":{"section_sources":[{"key":"fulltext","label":"Full Text","sources":["PMC"]}]}}'
echo "$pdf_json" | mustmatch like '{"_meta":{"section_sources":[{"key":"fulltext","label":"Full Text","sources":["Semantic Scholar"]}]}}'
```

## Article Fulltext Resolver Order

BioMCP may first bridge PMID or DOI identifiers to PMCID, then tries fulltext
sources in a fixed order. The Semantic Scholar PDF rung is attempted only when
the caller explicitly opts in with `--pdf`.

```bash
bin="${BIOMCP_BIN:-$(git rev-parse --show-toplevel)/target/release/biomcp}"
mkdir -p "$PWD/.cache"
fixture_workspace="$(mktemp -d "$PWD/.cache/article-fulltext-order.XXXXXX")"
trap 'rm -rf "$fixture_workspace"' EXIT
bash fixtures/setup-article-fulltext-source-fixture.sh "$fixture_workspace"
. "$fixture_workspace/.cache/spec-article-fulltext-source-env"
: > "$BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_REQUEST_LOG"
out="$("$bin" get article 22663014 fulltext --pdf)"
echo "$out" | mustmatch like "XML, HTML, and PDF sources did not return full text"
cat > "$fixture_workspace/expected-order.txt" <<'EOF'
fulltext:identity:ncbi-idconv
fulltext:xml:europepmc-pmc
fulltext:xml:ncbi-efetch-pmc
fulltext:xml:pmc-oa-archive
fulltext:xml:europepmc-med
fulltext:html:pmc
fulltext:pdf:semantic-scholar
EOF
grep '^fulltext:' "$BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_REQUEST_LOG" > "$fixture_workspace/actual-order.txt"
diff -u "$fixture_workspace/expected-order.txt" "$fixture_workspace/actual-order.txt"
```

## JSON section_sources — Gene, Drug, Disease

Core entity types must include a non-empty `_meta.section_sources` array. The
gene markdown and JSON assertions intentionally stay paired in the earlier
`get gene CFTR all` block so the spec gate proves the combined runtime path.

```bash
bin="${BIOMCP_BIN:-biomcp}"
drug_safety_json="$("$bin" get drug ivacaftor safety --json)"
echo "$drug_safety_json" | mustmatch like '"section_sources": ['
echo "$drug_safety_json" | mustmatch like '"key": "safety"'
echo "$drug_safety_json" | mustmatch like "OpenFDA FAERS"
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
drug_targets_json="$("$bin" get drug tamoxifen targets --json)"
echo "$drug_targets_json" | mustmatch like '"section_sources": ['
echo "$drug_targets_json" | mustmatch like '"key": "targets"'
echo "$drug_targets_json" | mustmatch like '"label": "ChEMBL"'
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
variant_drug_json="$("$bin" get drug rindopepimut targets --json)"
echo "$variant_drug_json" | mustmatch like '"key": "variant_targets"'
echo "$variant_drug_json" | mustmatch like '"label": "Variant Targets"'
echo "$variant_drug_json" | mustmatch like '"sources": ['
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
disease_json="$("$bin" get disease "cystic fibrosis" --json)"
echo "$disease_json" | mustmatch like '"section_sources": ['
echo "$disease_json" | mustmatch like '"key": "definition"'
echo "$disease_json" | mustmatch like '"key": "top_genes"'
echo "$disease_json" | mustmatch like '"key": "recruiting_trials"'
echo "$disease_json" | mustmatch like "MyDisease.info"
echo "$disease_json" | mustmatch like "ClinicalTrials.gov"
```

## JSON section_sources — Diagnostic, Variant, Trial, Article

```bash
bin="${BIOMCP_BIN:-$(git rev-parse --show-toplevel)/target/release/biomcp}"
bash fixtures/setup-gtr-spec-fixture.sh "$PWD"
bash fixtures/setup-who-ivd-spec-fixture.sh "$PWD"
. "$PWD/.cache/spec-gtr-env"
. "$PWD/.cache/spec-who-ivd-env"
gtr_diag_json="$("$bin" get diagnostic GTR000000001.1 genes --json)"
echo "$gtr_diag_json" | mustmatch like '"section_sources": ['
echo "$gtr_diag_json" | mustmatch like '"key": "summary"'
echo "$gtr_diag_json" | mustmatch like '"key": "genes"'
echo "$gtr_diag_json" | mustmatch like "NCBI Genetic Testing Registry"

who_diag_json="$("$bin" get diagnostic 'ITPW02232- TC40' conditions --json)"
echo "$who_diag_json" | mustmatch like '"section_sources": ['
echo "$who_diag_json" | mustmatch like '"key": "summary"'
echo "$who_diag_json" | mustmatch like '"key": "conditions"'
echo "$who_diag_json" | mustmatch like "WHO Prequalified IVD"
```

## JSON section_sources — Diagnostic Regulatory

```bash
bin="${BIOMCP_BIN:-$(git rev-parse --show-toplevel)/target/release/biomcp}"
bash fixtures/setup-gtr-spec-fixture.sh "$PWD"
. "$PWD/.cache/spec-gtr-env"
gtr_regulatory_json="$("$bin" get diagnostic GTR000000001.1 regulatory --json)"
echo "$gtr_regulatory_json" | mustmatch like '"regulatory": ['
echo "$gtr_regulatory_json" | mustmatch like '"section_sources": ['
echo "$gtr_regulatory_json" | mustmatch like '"key": "regulatory"'
echo "$gtr_regulatory_json" | mustmatch like '"label": "Regulatory"'
echo "$gtr_regulatory_json" | mustmatch like 'OpenFDA Device 510(k) / PMA'
```

```bash
variant_json="$(biomcp get variant rs334 --json)"
echo "$variant_json" | mustmatch like '"section_sources": ['
echo "$variant_json" | mustmatch like '"key": "identity"'
echo "$variant_json" | mustmatch like "MyVariant.info"
```

```bash

trial_json="$(biomcp get trial NCT06668103 --json)"
echo "$trial_json" | mustmatch like '"section_sources": ['
echo "$trial_json" | mustmatch like '"key": "overview"'
echo "$trial_json" | mustmatch like "ClinicalTrials.gov"
```

```bash

article_json="$(biomcp get article 22663011 --json)"
echo "$article_json" | mustmatch like '"section_sources": ['
echo "$article_json" | mustmatch like '"key": "bibliography"'
echo "$article_json" | mustmatch like '"label": "PubMed"'
```

## JSON section_sources — Pathway, Protein, PGX, Adverse Event

Reactome pathway cards are slower than the other entity types in this group, so
the identity proof runs on its own to stay within the shared spec timeout while
still checking the same `_meta.section_sources` contract.

```bash
pathway_json="$(biomcp get pathway R-HSA-5358351 --json)"
echo "$pathway_json" | mustmatch like '"section_sources": ['
echo "$pathway_json" | mustmatch like '"key": "identity"'
echo "$pathway_json" | mustmatch like '"label": "Reactome"'
```

The remaining entity families respond quickly enough to keep in one block while
still verifying their identity and section-level source labels.

```bash
wp_json="$(biomcp get pathway WP254 --json)"
echo "$wp_json" | mustmatch like '"section_sources": ['
echo "$wp_json" | mustmatch like '"key": "identity"'
echo "$wp_json" | mustmatch like "WikiPathways"

protein_json="$(biomcp get protein P15056 --json)"
echo "$protein_json" | mustmatch like '"section_sources": ['
echo "$protein_json" | mustmatch like '"key": "identity"'
echo "$protein_json" | mustmatch like '"label": "UniProt"'

pgx_json="$(biomcp get pgx CYP2D6 --json)"
echo "$pgx_json" | mustmatch like '"section_sources": ['
echo "$pgx_json" | mustmatch like '"label": "CPIC"'

ae_json="$(biomcp get adverse-event 10329882 --json)"
echo "$ae_json" | mustmatch like '"section_sources": ['
echo "$ae_json" | mustmatch like '"key": "reactions"'
echo "$ae_json" | mustmatch like '"label": "OpenFDA"'
```

## Backward Compatibility

Adding `section_sources` must not break the existing `_meta` contract.

```bash
gene_json="$(biomcp get gene CFTR --json)"
echo "$gene_json" | mustmatch like '"evidence_urls": ['
echo "$gene_json" | mustmatch like '"next_commands": ['
echo "$gene_json" | mustmatch like '"section_sources": ['
```
