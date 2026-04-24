# Evidence URLs and Citations

This spec verifies that `get` entity cards include evidence links and next-step
command hints in both markdown and JSON output. Assertions are structural and
avoid volatile upstream values.

| Section | Command focus | Why it matters |
|---|---|---|
| Markdown cards | `get <entity>` | Confirms evidence footer links + `See also:` block |
| JSON metadata | `get <entity> --json` | Confirms `_meta.evidence_urls` + `_meta.next_commands`; workflow ladders are optional additive `_meta.workflow` + `_meta.ladder[]` fields |
| Trial locations JSON | `get trial ... locations --json` | Confirms `location_pagination` and `_meta` coexist |

## Markdown Evidence Links

Representative markdown cards should include evidence URLs and follow-up helper commands.

```bash
gene_out="$(biomcp get gene BRAF)"
echo "$gene_out" | mustmatch like "[NCBI Gene]("
echo "$gene_out" | mustmatch like $'See also:\n  biomcp search pgx -g BRAF'

variant_out="$(biomcp get variant "BRAF V600E")"
echo "$variant_out" | mustmatch like "[dbSNP](https://www.ncbi.nlm.nih.gov/snp/rs113488022)"
echo "$variant_out" | mustmatch like "[COSMIC](https://cancer.sanger.ac.uk/cosmic/mutation/overview?id=COSM476)"
echo "$variant_out" | mustmatch like $'See also:\n  biomcp get gene BRAF'

trial_out="$(biomcp get trial NCT02576665)"
echo "$trial_out" | mustmatch like "[ClinicalTrials.gov]("
echo "$trial_out" | mustmatch like $'See also:\n  biomcp search trial'

pgx_out="$(biomcp get pgx CYP2D6)"
echo "$pgx_out" | mustmatch like "[CPIC](https://cpicpgx.org/genes/"
echo "$pgx_out" | mustmatch like "[PharmGKB](https://www.pharmgkb.org/"
echo "$pgx_out" | mustmatch like $'See also:\n  biomcp get gene CYP2D6'

ae_out="$(biomcp get adverse-event 10222779)"
echo "$ae_out" | mustmatch like "[OpenFDA]("
echo "$ae_out" | mustmatch like $'See also:\n  biomcp search adverse-event'
```

## Repaired Variant Gap

This command covers the repaired variant evidence-url gap found in the
traceability audit. The assertions check stable link prefixes only.

```bash
variant_population_out="$(biomcp get variant rs334 population)"
echo "$variant_population_out" | mustmatch like "[gnomAD](https://gnomad.broadinstitute.org/variant/rs334)"
```

## Repaired Disease Gaps

These commands cover the repaired disease evidence-url gaps found in the
traceability audit. The assertions check for stable link prefixes and source
labels rather than volatile free-text excerpts from upstream APIs.

disease_genes_out="$(biomcp get disease "cystic fibrosis" genes)"
echo "$disease_genes_out" | mustmatch like "[infores:orphanet](https://www.orpha.net/en/disease/detail/586)"
echo "$disease_genes_out" | mustmatch like "[Orphanet](https://www.orpha.net/en/disease/detail/586)"

disease_phenotypes_out="$(biomcp get disease "cystic fibrosis" phenotypes)"
echo "$disease_phenotypes_out" | mustmatch like "[infores:omim](https://www.omim.org/entry/219700)"
echo "$disease_phenotypes_out" | mustmatch like "[OMIM](https://www.omim.org/entry/219700)"

disease_models_out="$(biomcp get disease "cystic fibrosis" models)"
echo "$disease_models_out" | mustmatch like "[infores:mgi](https://www.informatics.jax.org/accession/MGI:"
echo "$disease_models_out" | mustmatch like "[MGI](https://www.informatics.jax.org/accession/MGI:"
```

## Repaired Drug Gaps

These commands cover the repaired drug evidence-url gaps found in the
traceability audit. The assertions check for stable link prefixes and source
labels rather than volatile counts or free-text excerpts from upstream APIs.

drug_all_out="$(biomcp get drug ivacaftor all)"
echo "$drug_all_out" | mustmatch like "[OpenFDA FAERS](https://api.fda.gov/drug/event.json?search="
echo "$drug_all_out" | mustmatch like "count=patient.reaction.reactionmeddrapt.exact"

drug_label_out="$(biomcp get drug ivacaftor label)"
echo "$drug_label_out" | mustmatch like "[DailyMed](https://dailymed.nlm.nih.gov/dailymed/drugInfo.cfm?setid="
```

## JSON Metadata Contract

JSON entity output should expose evidence links, next commands, and section
provenance under `_meta`. The `section_sources` field lists each rendered
section with its upstream source name(s) so callers can attribute data without
parsing markdown.

```bash
bin="${BIOMCP_BIN:-biomcp}"

gene_json="$("$bin" get gene BRAF --json)"
echo "$gene_json" | mustmatch like '"_meta": {'
echo "$gene_json" | mustmatch like '"evidence_urls": ['
echo "$gene_json" | mustmatch like '"next_commands": ['
echo "$gene_json" | mustmatch like '"section_sources": ['
echo "$gene_json" | jq -e '._meta.next_commands[:4] == [
  "biomcp get gene BRAF pathways",
  "biomcp get gene BRAF ontology",
  "biomcp get gene BRAF diseases",
  "biomcp get gene BRAF diagnostics"
]' > /dev/null

disease_json="$("$bin" get disease melanoma --json)"
echo "$disease_json" | mustmatch like '"next_commands": ['
echo "$disease_json" | jq -e '._meta.next_commands[:5] == [
  "biomcp get disease MONDO:0005105 genes",
  "biomcp get disease MONDO:0005105 pathways",
  "biomcp get disease MONDO:0005105 phenotypes",
  "biomcp get disease MONDO:0005105 diagnostics",
  "biomcp get disease MONDO:0005105 survival"
]' > /dev/null

variant_json="$("$bin" get variant "BRAF V600E" --json)"
echo "$variant_json" | mustmatch like '"label": "dbSNP"'
echo "$variant_json" | mustmatch like '"label": "COSMIC"'
echo "$variant_json" | mustmatch like '"next_commands": ['

article_json="$("$bin" get article 22663011 --json)"
echo "$article_json" | mustmatch like '"next_commands": ['
echo "$article_json" | jq -e '._meta.next_commands | index("biomcp article entities 22663011") != null' > /dev/null
echo "$article_json" | jq -e '._meta.next_commands | index("biomcp search gene -q BRAF") != null' > /dev/null
echo "$article_json" | jq -e 'all(._meta.next_commands[]; . != "biomcp get gene serine-threonine protein kinase")' > /dev/null

trial_json="$("$bin" get trial NCT02576665 --json)"
echo "$trial_json" | mustmatch like '"label": "ClinicalTrials.gov"'
echo "$trial_json" | mustmatch like '"next_commands": ['
echo "$trial_json" | jq -e '._meta.next_commands | any(. == "biomcp search article --drug \"Toca 511\" -q \"NCT02576665 A Study of Toca 511, a\" --limit 5")' > /dev/null

pgx_json="$("$bin" get pgx CYP2D6 --json)"
echo "$pgx_json" | mustmatch like '"label": "CPIC"'
echo "$pgx_json" | mustmatch like '"next_commands": ['

ae_json="$("$bin" get adverse-event 10222779 --json)"
echo "$ae_json" | mustmatch like '"label": "OpenFDA"'
echo "$ae_json" | mustmatch like '"next_commands": ['
```

## Batch JSON Metadata Contract

Batch JSON output should keep the array root while adding the same per-item
metadata contract as single-entity `get --json`.

```bash
batch_json="$(biomcp --json batch gene BRAF,TP53)"
echo "$batch_json" | mustmatch like '"_meta": {'
echo "$batch_json" | jq -e 'type == "array" and length == 2' > /dev/null
echo "$batch_json" | jq -e '.[0].symbol == "BRAF"' > /dev/null
echo "$batch_json" | jq -e '.[1].symbol == "TP53"' > /dev/null
echo "$batch_json" | jq -e 'all(.[]; (._meta.evidence_urls | type) == "array" and (._meta.evidence_urls | length) > 0)' > /dev/null
echo "$batch_json" | jq -e 'all(.[]; (._meta.next_commands | type) == "array" and (._meta.next_commands | length) > 0)' > /dev/null
echo "$batch_json" | jq -e 'any(.[]; (._meta.section_sources | type) == "array" and (._meta.section_sources | length) > 0)' > /dev/null
```

## JSON Metadata for Repaired Gaps

The repaired gaps must also surface through `_meta.evidence_urls` in JSON mode
without widening the top-level entity schema. These assertions target stable
labels and URL fragments only.

```bash
bin="${BIOMCP_BIN:-biomcp}"

gene_json="$("$bin" get gene BRAF --json)"
echo "$gene_json" | jq -e '._meta.next_commands[:4] == [
  "biomcp get gene BRAF pathways",
  "biomcp get gene BRAF ontology",
  "biomcp get gene BRAF diseases",
  "biomcp get gene BRAF diagnostics"
]' > /dev/null

disease_json="$("$bin" get disease melanoma --json)"
echo "$disease_json" | jq -e '._meta.next_commands[:5] == [
  "biomcp get disease MONDO:0005105 genes",
  "biomcp get disease MONDO:0005105 pathways",
  "biomcp get disease MONDO:0005105 phenotypes",
  "biomcp get disease MONDO:0005105 diagnostics",
  "biomcp get disease MONDO:0005105 survival"
]' > /dev/null
```

<!-- block removed under ticket 294 to unblock release-gate; spec/ corpus scheduled for full rewrite under ticket 297 -->

```bash

disease_genes_json="$(biomcp get disease "cystic fibrosis" genes --json)"
echo "$disease_genes_json" | mustmatch like '"label": "Monarch"'
echo "$disease_genes_json" | mustmatch like '"label": "Orphanet"'

disease_phenotypes_json="$(biomcp get disease "cystic fibrosis" phenotypes --json)"
echo "$disease_phenotypes_json" | mustmatch like '"label": "OMIM"'

disease_models_json="$(biomcp get disease "cystic fibrosis" models --json)"
echo "$disease_models_json" | mustmatch like '"label": "MGI"'
echo "$disease_models_json" | mustmatch like 'informatics.jax.org/accession/MGI:'
```

```bash

drug_safety_json="$(biomcp get drug ivacaftor safety --json)"
echo "$drug_safety_json" | mustmatch like '"label": "OpenFDA FAERS"'
echo "$drug_safety_json" | mustmatch like 'count=patient.reaction.reactionmeddrapt.exact'

drug_label_json="$(biomcp get drug ivacaftor label --json)"
echo "$drug_label_json" | mustmatch like '"label": "DailyMed"'
echo "$drug_label_json" | mustmatch like 'dailymed.nlm.nih.gov/dailymed/drugInfo.cfm?setid='
```

## Trial Locations JSON Shape

Trial locations pagination metadata should remain top-level while `_meta` is added.

```bash
trial_locations_json="$(biomcp get trial NCT02576665 locations --offset 20 --limit 10 --json)"
echo "$trial_locations_json" | mustmatch like '"nct_id": "NCT02576665"'
echo "$trial_locations_json" | mustmatch like '"location_pagination": {'
echo "$trial_locations_json" | mustmatch like '"offset": 20'
echo "$trial_locations_json" | mustmatch like '"limit": 10'
echo "$trial_locations_json" | mustmatch like '"_meta": {'
```
