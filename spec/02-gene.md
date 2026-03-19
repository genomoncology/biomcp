# Gene Queries

Genes are a primary anchor in BioMCP and frequently drive downstream trial, article, and drug exploration. These checks verify search/get behavior and helper commands using structural output invariants. The intent is to keep the assertions robust across changing source records.

| Section | Command focus | Why it matters |
|---|---|---|
| Symbol search | `search gene BRAF` | Confirms canonical gene lookup |
| Table structure | `search gene BRAF` | Confirms stable result schema |
| Detail card | `get gene BRAF` | Confirms rich per-gene card output |
| Section expansion | `get gene BRAF pathways` | Confirms progressive disclosure |
| HPA section | `get gene BRAF hpa` | Confirms protein tissue-expression contract |
| Druggability section | `get gene EGFR druggability` | Confirms combined DGIdb/OpenTargets contract |
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
echo "$out" | mustmatch like "Use "
echo "$out" | mustmatch like "get gene <symbol>"
```

## Getting Gene Details

`get gene` should return a concise identity card with persistent identifiers. Entrez ID is a durable anchor for this entity.

```bash
out="$(biomcp get gene BRAF)"
echo "$out" | mustmatch like "# BRAF ("
echo "$out" | mustmatch like "Entrez ID: 673"
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
echo "$out" | mustmatch like "pLI:"
echo "$out" | mustmatch like "LOEUF:"
```

## Human Protein Atlas Section

The HPA section should expose protein tissue expression, localization context, and stable HPA labels without dumping the raw upstream record.

```bash
out="$(biomcp get gene BRAF hpa)"
echo "$out" | mustmatch like "## Human Protein Atlas"
echo "$out" | mustmatch like "Reliability:"
echo "$out" | mustmatch like "Subcellular"
echo "$out" | mustmatch like "| Tissue | Level |"
```

## Druggability Section

The druggability section should stay as one section while exposing OpenTargets tractability markers and safety-liability context alongside DGIdb interaction data.

```bash
out="$(biomcp get gene EGFR druggability)"
echo "$out" | mustmatch like "## Druggability"
echo "$out" | mustmatch like "OpenTargets tractability"
echo "$out" | mustmatch like "small molecule"
echo "$out" | mustmatch like "antibody"
echo "$out" | mustmatch like "OpenTargets safety liabilities"
```

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
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search gene ERBB1 --limit 5)"
echo "$out" | mustmatch like "# Genes: ERBB1"
echo "$out" | mustmatch like "EGFR"
```

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search gene P53 --limit 5)"
echo "$out" | mustmatch like "# Genes: P53"
echo "$out" | mustmatch like "TP53"
```

## Gene DisGeNET Associations

DisGeNET scored gene-disease associations require `DISGENET_API_KEY`. The section heading and table schema are stable invariants; individual scores and row counts vary by API tier.

```bash
out="$(biomcp get gene TP53 disgenet)"
echo "$out" | mustmatch like "## DisGeNET"
echo "$out" | mustmatch like "| Disease | UMLS CUI | Score | PMIDs | Trials | EL | EI |"
```

```bash
out="$(biomcp get gene TP53 disgenet --json)"
echo "$out" | mustmatch json ".disgenet.associations | length > 0"
```
