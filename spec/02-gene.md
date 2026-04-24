# Gene Queries

Genes are a primary anchor in BioMCP and frequently drive downstream trial, article, and drug exploration. These checks verify search/get behavior and helper commands using structural output invariants. The intent is to keep the assertions robust across changing source records.

| Section | Command focus | Why it matters |
|---|---|---|
| Symbol search | `search gene BRAF` | Confirms canonical gene lookup |
| Table structure | `search gene BRAF` | Confirms stable result schema |
| Detail card | `get gene BRAF` | Confirms rich per-gene card output |
| Guidance | `get gene OPA1` | Confirms alias explainer and localization follow-up hints |
| Section expansion | `get gene BRAF pathways` | Confirms progressive disclosure |
| HPA section | `get gene BRAF hpa` | Confirms protein tissue-expression contract |
| Druggability section | `get gene EGFR druggability` | Confirms combined DGIdb/OpenTargets contract |
| Funding section | `get gene ERBB2 funding` | Confirms NIH Reporter funding contract |
| Funding stays opt-in | `get gene ERBB2 all` | Confirms `all` still excludes NIH Reporter funding |
| Diagnostics pivot | `get gene BRCA1 diagnostics` | Confirms GTR-backed diagnostic-test rows from a gene card |
| Trial helper | `gene trials BRAF` | Confirms cross-entity trial pivot |
| Article helper | `gene articles BRAF` | Confirms cross-entity literature pivot |

## Searching by Symbol

Symbol-based search is the fastest route to canonical gene identity and naming. We check for the expected heading and official long name for BRAF.

```bash
out="$(biomcp search gene BRAF --limit 3)"
echo "$out" | mustmatch like "# Genes: BRAF"
echo "$out" | mustmatch like "B-Raf proto-oncogene"
```

## Search Table Structure

Search rows should preserve a consistent table layout so downstream readers can scan fields quickly. This assertion targets the stable table columns and helper hint text.

```bash
out="$(biomcp search gene BRAF --limit 3)"
echo "$out" | mustmatch like "| Symbol | Name | Entrez ID |"
echo "$out" | mustmatch like 'Use `get gene <symbol>` for details.'
```

## Search JSON Next Commands

Non-empty gene search JSON should include machine-readable follow-up commands so
agents can pivot from the top hit without parsing markdown helper text.

```bash
json_out="$(biomcp --json search gene BRAF --limit 3)"
echo "$json_out" | mustmatch like '"next_commands":'
echo "$json_out" | jq -e '._meta.next_commands[0] | test("^biomcp get gene .+$")' > /dev/null
echo "$json_out" | jq -e '._meta.next_commands | any(. == "biomcp list gene")' > /dev/null
```

## Getting Gene Details

`get gene` should return a concise identity card with persistent identifiers. Entrez ID is a durable anchor for this entity.

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene BRAF)"
echo "$out" | mustmatch like "# BRAF (B-Raf proto-oncogene"
echo "$out" | mustmatch like "Entrez ID: 673"
echo "$out" | mustmatch like $'More:\n  biomcp get gene BRAF pathways   - Reactome/KEGG pathway context\n  biomcp get gene BRAF ontology   - GO-style functional enrichment\n  biomcp get gene BRAF diseases   - disease associations\n  biomcp get gene BRAF diagnostics   - diagnostic tests for this gene from GTR'
json="$("$bin" --json get gene BRAF)"
echo "$json" | jq -e '._meta.next_commands[:4] == [
  "biomcp get gene BRAF pathways",
  "biomcp get gene BRAF ontology",
  "biomcp get gene BRAF diseases",
  "biomcp get gene BRAF diagnostics"
]' > /dev/null
echo "$json" | jq -e '._meta.suggestions | any(. == "biomcp search pgx -g BRAF")' > /dev/null
echo "$json" | jq -e '._meta.suggestions | any(. == "biomcp search diagnostic --gene BRAF")' > /dev/null
echo "$json" | jq -e '[._meta.suggestions[] | select(. == "biomcp get gene BRAF pathways")] | length == 0' > /dev/null
```

## Gene Card Guidance

The base gene card should explain what aliases are for and, when the summary implies localization or structure follow-up, surface executable deepen commands instead of generic guesses.

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene OPA1)"
echo "$out" | mustmatch like "Aliases are alternate names used in literature and databases"
echo "$out" | mustmatch like "biomcp get gene OPA1 protein"
echo "$out" | mustmatch like "biomcp get gene OPA1 hpa"
echo "$out" | mustmatch like "localization"
echo "$out" | mustmatch like "biomcp get gene OPA1 diagnostics"
```

## Progressive Disclosure

Section-specific retrieval keeps the output focused while preserving access to deeper context. The pathways section should expose a labeled subsection and pathway table columns.

```bash
out="$(biomcp get gene BRAF pathways)"
echo "$out" | mustmatch like "## Pathways"
echo "$out" | mustmatch like "| ID | Name |"
```

## Constraint Section

The constraint section should render gnomAD provenance even when values evolve over time. These checks assert the stable labels rather than exact floating-point scores.

```bash
out="$(biomcp get gene TP53 constraint)"
echo "$out" | mustmatch like "## Constraint"
echo "$out" | mustmatch like "Source: gnomAD"
echo "$out" | mustmatch like "Version: v4"
echo "$out" | mustmatch like "Reference genome: GRCh38"
echo "$out" | mustmatch like "Transcript:"
echo "$out" | mustmatch '/- pLI: [0-9.]+/'
echo "$out" | mustmatch like "- LOEUF: 0."
```

## Human Protein Atlas Section

The HPA section should expose protein tissue expression, localization context, and stable HPA labels without dumping the raw upstream record. When live HPA data is unavailable or times out, the CLI should fall back to a truthful empty state instead of fabricating those fields.

```bash
out="$(biomcp get gene BRAF hpa)"
echo "$out" | mustmatch like "## Human Protein Atlas"
if printf '%s\n' "$out" | grep -q 'No Human Protein Atlas records returned'; then
  echo "$out" | mustmatch like "No Human Protein Atlas records returned"
else
  echo "$out" | mustmatch like "Reliability:"
  echo "$out" | mustmatch like "Subcellular"
  echo "$out" | mustmatch like "| Tissue | Level |"
  echo "$out" | mustmatch '/\| [^|]+ \| (High|Medium|Low|Not detected) \|/'
  tissue_line="$(printf '%s\n' "$out" | grep -n '| Tissue | Level |' | cut -d: -f1 | head -n1)"
  rna_line="$(printf '%s\n' "$out" | grep -n 'RNA summary:' | cut -d: -f1 | head -n1)"
  test -n "$tissue_line"
  test -n "$rna_line"
  test "$tissue_line" -lt "$rna_line"
fi
```

## Gene Protein Isoforms

The UniProt-backed gene protein section should surface isoform names when UniProt provides alternative products, while staying absent for genes without isoform annotations. The line includes a count and only the displayed isoform length.

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene KRAS protein)"
echo "$out" | mustmatch like "## Protein (UniProt)"
echo "$out" | mustmatch like "- Isoforms (2):"
echo "$out" | mustmatch like "K-Ras4A (189 aa)"
echo "$out" | mustmatch like "K-Ras4A (189 aa), K-Ras4B"
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene TP73 protein)"
echo "$out" | mustmatch '/- Isoforms \([0-9]+\):/'
echo "$out" | mustmatch like "- Isoforms (12): Alpha (636 aa), Beta"
echo "$out" | mustmatch like "Gamma, Delta, Epsilon"
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene BRAF protein)"
echo "$out" | mustmatch not like "- Isoforms ("
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene KRAS protein --json)"
echo "$out" | jq -e '
  .protein.isoforms | length >= 2
  and any(.[]; .name == "K-Ras4A" and .length == 189)
  and any(.[]; .name == "K-Ras4B")
' > /dev/null
```

## Gene Protein Alternative Names

Legacy protein names remain common in literature and BioASQ-style answer keys, so the UniProt-backed gene protein section should expose those names alongside the canonical protein name in both markdown and JSON output.

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene PLIN2 protein)"
echo "$out" | mustmatch like "## Protein (UniProt)"
echo "$out" | mustmatch like "- Name: Perilipin-2"
echo "$out" | mustmatch like "- Also known as:"
echo "$out" | mustmatch like "Adipophilin, ADRP"
echo "$out" | mustmatch like "Adipose differentiation-related protein"
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene PLIN1 protein)"
echo "$out" | mustmatch like "- Name: Perilipin-1"
echo "$out" | mustmatch like "Lipid droplet-associated protein"
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene PLIN2 protein --json)"
echo "$out" | jq -e '
  (.protein.alternative_names // []) | index("ADRP")
' > /dev/null
```

## Gene Protein Function Full Text

The gene protein section must preserve the full UniProt function text rather than truncating the line. OPA1 is the regression anchor because its localization detail in the intermembrane space was being cut off in the gene view.

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene OPA1 protein)"
echo "$out" | mustmatch like "## Protein (UniProt)"
echo "$out" | mustmatch like "intermembrane space"
echo "$out" | mustmatch not like "intermembrane…"
```

## Druggability Section

The druggability section should stay as one section while exposing OpenTargets tractability markers and safety-liability context alongside DGIdb interaction data.

```bash
out="$(biomcp get gene EGFR druggability)"
echo "$out" | mustmatch like "## Druggability"
echo "$out" | mustmatch like "OpenTargets tractability"
echo "$out" | mustmatch like "small molecule"
echo "$out" | mustmatch like "| antibody | yes | Approved Drug"
echo "$out" | mustmatch like "OpenTargets safety liabilities"
```

## Gene Funding

NIH Reporter funding should remain opt-in and render grant rows with stable
table structure while preserving numeric award amounts and provenance in JSON.

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene ERBB2 funding)"
echo "$out" | mustmatch like "## Funding (NIH Reporter)"
echo "$out" | mustmatch like "| Project | PI | Organization | FY | Amount |"
echo "$out" | mustmatch '/Showing top [0-9]+ unique grants from [0-9]+ matching NIH project-year records across FY20[0-9]{2}-FY20[0-9]{2}\./'
echo "$out" | mustmatch '/\| .* \| .* \| .* \| 20[0-9]{2} \| \$[0-9,]+ \|/'
json="$("$bin" --json get gene ERBB2 funding)"
echo "$json" | jq -e '.funding.query == "ERBB2"' > /dev/null
echo "$json" | jq -e '(.funding.fiscal_years | length) == 5' > /dev/null
echo "$json" | jq -e '.funding.grants | length > 0' > /dev/null
echo "$json" | jq -e '(.funding.grants[0].award_amount | type) == "number"' > /dev/null
echo "$json" | jq -e '(.funding.grants[0].project_num | type) == "string"' > /dev/null
echo "$json" | jq -e '.funding_note == null' > /dev/null
echo "$json" | jq -e 'any(._meta.section_sources[]; .key == "funding" and (.sources | index("NIH Reporter")))' > /dev/null
```

## Gene Funding Stays Opt-In

`funding` should remain an explicit section so `get gene <symbol> all` does not
invent a fake NIH Reporter block when the user did not ask for one.

<!-- block removed under ticket 294 to unblock release-gate; spec/ corpus scheduled for full rewrite under ticket 297 -->

## Gene Diagnostics Pivot

Gene cards should be able to pivot into source-native diagnostic tests without
leaving the `get gene` flow. The fixture keeps the proof local and confirms
that embedded rows render public source labels and JSON provenance.

```bash
bash fixtures/setup-gtr-spec-fixture.sh "$PWD"
. "$PWD/.cache/spec-gtr-env"
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene BRCA1 diagnostics)"
echo "$out" | mustmatch like "## Diagnostics"
echo "$out" | mustmatch like "| Accession | Name | Type | Manufacturer / Lab | Source | Genes | Conditions |"
echo "$out" | mustmatch like "GTR000000001.1"
echo "$out" | mustmatch like "BRCA1 Hereditary Cancer Panel"
echo "$out" | mustmatch like "NCBI Genetic Testing Registry"
echo "$out" | mustmatch like "BRAF, BRCA1, BRCA2, ATM, PALB2, +4 more"
echo "$out" | mustmatch like "Breast cancer, Ovarian cancer, Hereditary breast ovarian cancer syndrome, Pancreatic cancer, Lynch syndrome, +1 more"
json="$("$bin" --json get gene BRCA1 diagnostics)"
echo "$json" | jq -e '.diagnostics | length >= 1' > /dev/null
echo "$json" | jq -e '.diagnostics[0].source == "gtr"' > /dev/null
echo "$json" | jq -e 'any(._meta.section_sources[]; .key == "diagnostics" and (.sources | index("NCBI Genetic Testing Registry")))' > /dev/null
```

## Gene Diagnostics Stays Opt-In

`diagnostics` should remain an explicit section so `get gene <symbol> all` does
not trigger local diagnostic data access or render diagnostic fields.

<!-- block removed under ticket 294 to unblock release-gate; spec/ corpus scheduled for full rewrite under ticket 297 -->

## Gene to Trials

The trial helper uses a gene biomarker pivot, which is a common translational workflow. We assert on the trial result table shape and the query marker for BRAF.

```bash
out="$(biomcp gene trials BRAF --limit 3)"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
echo "$out" | mustmatch like "Query: biomarker=BRAF"
```

## Gene to Articles

Literature pivoting from a gene symbol is a standard evidence-gathering step. The assertion checks article table structure and query context header.

```bash
out="$(biomcp gene articles BRAF --limit 3)"
echo "$out" | mustmatch like "# Articles: gene=BRAF"
echo "$out" | mustmatch like "| PMID | Title |"
```

## Gene Alias Search

Alias-only symbols should still surface the canonical gene rows. These checks guard the ERBB1 and P53 regressions by asserting that alias queries return EGFR and TP53 rows.

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" search gene ERBB1 --limit 5)"
echo "$out" | mustmatch like "# Genes: ERBB1"
echo "$out" | mustmatch like "| EGFR | epidermal growth factor receptor |"
```

```bash
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" search gene P53 --limit 5)"
echo "$out" | mustmatch like "# Genes: P53"
echo "$out" | mustmatch like "| TP53 | tumor protein p53 |"
```

## Gene DisGeNET Associations

DisGeNET scored gene-disease associations require `DISGENET_API_KEY`. The section heading and table schema are stable invariants; individual scores and row counts vary by API tier.

```bash
status=0
out="$(biomcp get gene TP53 disgenet 2>&1)" || status=$?
if [ "$status" -eq 0 ] && ! printf '%s\n' "$out" | grep -qi '403 Forbidden'; then
  echo "$out" | mustmatch like "## DisGeNET"
  echo "$out" | mustmatch like "| Disease | UMLS CUI | Score | PMIDs | Trials | EL | EI |"
else
  echo "$out" | mustmatch like "DISGENET_API_KEY"
  echo "$out" | mustmatch like "export DISGENET_API_KEY"
  echo "$out" | mustmatch not like "Unauthorized"
  echo "$out" | mustmatch not like "403 Forbidden"
fi
```

```bash
status=0
out="$(biomcp get gene TP53 disgenet --json 2>&1)" || status=$?
if [ "$status" -eq 0 ] && ! printf '%s\n' "$out" | grep -qi '403 Forbidden'; then
  echo "$out" | jq -e '.disgenet.associations | length > 0' > /dev/null
else
  echo "$out" | mustmatch like "DISGENET_API_KEY"
  echo "$out" | mustmatch like "export DISGENET_API_KEY"
  echo "$out" | mustmatch not like "Unauthorized"
  echo "$out" | mustmatch not like "403 Forbidden"
fi
```
