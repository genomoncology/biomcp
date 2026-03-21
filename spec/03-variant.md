# Variant Queries

Variant workflows in BioMCP unify clinical significance, population frequency, and helper pivots to trials and literature. This file validates both broad search behavior and focused retrieval of BRAF V600E context. Assertions emphasize headings, columns, and known durable identifiers.

| Section | Command focus | Why it matters |
|---|---|---|
| Gene-level search | `search variant -g BRAF` | Confirms variant table rendering |
| Protein filter | `search variant -g BRAF --hgvsp V600E` | Confirms targeted query narrowing |
| Detail card | `get variant "BRAF V600E"` | Confirms consolidated variant facts |
| ClinVar section | `get variant ... clinvar` | Confirms clinical interpretation expansion |
| Population section | `get variant ... population` | Confirms frequency context expansion |
| Trial helper | `variant trials ...` | Confirms mutation-centric trial lookup |
| Article helper | `variant articles ...` | Confirms mutation-centric literature lookup |

## Searching by Gene

Gene-scoped variant search is the broad intake step before applying mutation-level filters. The output should include a stable column header set and explicit query echo.

```bash
out="$(biomcp search variant -g BRAF --limit 3)"
echo "$out" | mustmatch like "| ID | Gene | Protein |"
echo "$out" | mustmatch like "Query: gene=BRAF"
```

## Finding a Specific Variant

Adding `--hgvsp` should constrain the result set to a precise protein change. We check for the query marker and V600E appearance in rows.

```bash
out="$(biomcp search variant -g BRAF --hgvsp V600E --limit 3)"
echo "$out" | mustmatch like "hgvsp=V600E"
echo "$out" | mustmatch like "V600E"
```

## Residue Alias Search

Gene-scoped residue aliases should stay on the variant path instead of falling
through to condition text. The query echo should describe the dedicated alias
search honestly.

```bash
out="$(biomcp search variant "PTPN22 620W" --limit 5)"
echo "$out" | mustmatch like "gene=PTPN22"
echo "$out" | mustmatch like "residue_alias=620W"
```

## Residue Alias Search with Gene Flag

A residue alias supplied as a positional token alongside `--gene` should use the
same dedicated alias search path as the two-token positional form.

```bash
out="$(biomcp search variant -g PTPN22 620W --limit 5)"
echo "$out" | mustmatch like "gene=PTPN22"
echo "$out" | mustmatch like "residue_alias=620W"
```

## Protein Shorthand with Gene Context

Standalone protein shorthand becomes safe once the gene is already supplied in a
flag. The query echo should show the normal exact protein filter rather than an
ambiguous gene search.

```bash
out="$(biomcp search variant -g PTPN22 R620W --limit 5)"
echo "$out" | mustmatch like "gene=PTPN22"
echo "$out" | mustmatch like "hgvsp=R620W"
```

## Standalone Protein Shorthand Guidance

Without gene context, a shorthand like `R620W` is too ambiguous for automatic
typed search. BioMCP should return variant-specific next commands rather than
silently rewriting the query into a gene or condition search.

```bash
status=0
out="$(biomcp search variant R620W 2>&1)" || status=$?
test "${status}" -eq 1
echo "$out" | mustmatch like "without gene context"
echo "$out" | mustmatch like "biomcp search variant --hgvsp R620W --limit 10"
echo "$out" | mustmatch like "biomcp discover R620W"
```

## Getting Variant Details

The default variant card should include both human-readable and identifier-centric fields. We assert on a stable rsID and pathogenicity marker.

```bash
out="$(biomcp get variant "BRAF V600E")"
echo "$out" | mustmatch like "rs113488022"
echo "$out" | mustmatch like "Significance: Pathogenic"
```

## ClinVar Section

ClinVar-focused expansion is used for clinical interpretation and evidence traceability. The check targets section heading and variant metadata marker.

```bash
out="$(biomcp get variant "BRAF V600E" clinvar)"
echo "$out" | mustmatch like "## ClinVar"
echo "$out" | mustmatch like "Variant ID:"
```

## ClinVar Compact Disease Anchor

ClinVar output should expose the top ranked disease aggregate directly so agents can read the leading condition without re-parsing the full list.

```bash
out="$(biomcp --json get variant "BRAF V600E" clinvar)"
echo "$out" | jq -e '.top_disease.condition | type == "string"' > /dev/null
echo "$out" | jq -e '.top_disease.reports | type == "number"' > /dev/null
```

## Population Frequencies

Population frequency context helps distinguish rare versus common variation. We assert on population section labeling and gnomAD field presence.

```bash
out="$(biomcp get variant "BRAF V600E" population)"
echo "$out" | mustmatch like "## Population"
echo "$out" | mustmatch like "gnomAD AF"
```

## Population Compact Fields

Population JSON should expose both a stable raw field and a compact percentage string so agents can answer frequency questions without table parsing.

```bash
out="$(biomcp --json get variant "BRAF V600E" population)"
echo "$out" | jq -e '.allele_frequency_raw | type == "number"' > /dev/null
echo "$out" | jq -e '.allele_frequency_percent | type == "string"' > /dev/null
```

## Population Compact Markdown

The markdown population line should keep the raw AF and append the compact percentage inline in the same section.

```bash
out="$(biomcp get variant "BRAF V600E" population)"
echo "$out" | mustmatch like "gnomAD AF:"
echo "$out" | mustmatch like "%"
```

## GWAS Supporting PMIDs

GWAS JSON should expose ordered supporting PMIDs as a dedicated array without requiring agents to traverse the full GWAS rows.

```bash
out="$(biomcp --json get variant rs7903146 gwas)"
echo "$out" | jq -e '.supporting_pmids | type == "array"' > /dev/null
```

## Get with Residue Alias Guidance

`get variant` remains exact-only, so residue aliases should return recovery
guidance that keeps the user on the variant search path.

```bash
status=0
out="$(biomcp get variant "PTPN22 620W" 2>&1)" || status=$?
test "${status}" -eq 1
echo "$out" | mustmatch like "BioMCP could not map 'PTPN22 620W' to an exact variant."
echo "$out" | mustmatch like "biomcp search variant \"PTPN22 620W\" --limit 10"
echo "$out" | mustmatch like "biomcp search variant -g PTPN22 --limit 10"
```

## JSON Guidance Metadata

JSON error output for variant shorthand should expose the same high-level
contract as the alias fallback flow: structured `_meta.alias_resolution`,
ordered `_meta.next_commands`, and a non-zero exit.

```bash
status=0
out="$(biomcp --json get variant R620W)" || status=$?
test "${status}" -eq 1
echo "$out" | mustmatch like '"alias_resolution": {'
echo "$out" | mustmatch like '"kind": "protein_change_only"'
echo "$out" | mustmatch like '"requested_entity": "variant"'
echo "$out" | mustmatch like '"next_commands": ['
echo "$out" | mustmatch like '"biomcp discover R620W"'
```

## Variant to Trials

A mutation-to-trial pivot is central for translational use cases. The trial output should include the expected table schema and mutation query echo.

```bash
out="$(biomcp variant trials "BRAF V600E" --limit 3)"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
echo "$out" | mustmatch like "Query: mutation=BRAF V600E"
```

## Variant to Articles

Literature helper commands should retain both gene and keyword context in the heading. We verify header context and article table columns.

```bash
out="$(biomcp variant articles "BRAF V600E" --limit 3)"
echo "$out" | mustmatch like "# Articles: gene=BRAF, keyword=V600E"
echo "$out" | mustmatch like "| PMID | Title |"
```

## Searching by rsID

rsID positional queries should normalize to an exact rsID search instead of falling back to gene text. The query echo and returned BRAF row guard the rsID normalization fix.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search variant rs113488022 --limit 5)"
echo "$out" | mustmatch like "Query: rsid=rs113488022"
echo "$out" | mustmatch like "BRAF"
```

## Searching by c.HGVS

Gene plus c.HGVS shorthand should map to exact gene and coding-change filters. This regression checks the query echo and BRAF recall.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search variant "BRAF c.1799T>A" --limit 5)"
echo "$out" | mustmatch like "gene=BRAF"
echo "$out" | mustmatch like "hgvsc=c.1799T>A"
echo "$out" | mustmatch like "BRAF"
```

## Exon Deletion Phrase Search

Confirmed exon-deletion phrases should resolve to a gene-scoped consequence search rather than generic condition text. The query echo should show the normalized consequence filter.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search variant "EGFR Exon 19 Deletion" --limit 5)"
echo "$out" | mustmatch like "gene=EGFR"
echo "$out" | mustmatch like "consequence=inframe_deletion"
echo "$out" | mustmatch like "EGFR"
```
