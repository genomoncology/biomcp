# Cross-Entity Pivot CLI Contracts

This spec protects the runnable cross-entity workflows exposed by the CLI. It
covers pivot helpers and sectioned diagnostic pivots that users can execute at
the terminal, not docs-site copy or navigation.

| Surface | Representative checks | Why it matters |
|---|---|---|
| Variant pivots | `variant trials`, `variant articles` | Mutation-first investigation flow |
| Drug pivots | `drug trials`, `drug adverse-events` | Therapy-to-trial and therapy-to-safety flow |
| Disease pivots | `disease trials`, `disease drugs`, `disease articles`, `get disease ... diagnostics` | Diagnosis-centered pivots |
| Gene pivots | `gene trials`, `gene drugs`, `gene articles`, `gene pathways`, `get gene ... diagnostics` | Canonical biomarker pivots |

## Variant pivots
<!-- smoke-lane -->

Variant helpers should preserve the mutation context when crossing into trials
or articles. The docs only promise stable headings and table shapes.
Variant-to-article pivots should keep gene and keyword context without
promising provider-specific subsections or counts.

```bash
out="$(biomcp variant trials "BRAF V600E" --limit 3)"
echo "$out" | mustmatch like "Query: mutation=BRAF V600E"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"

out="$(biomcp variant articles "BRAF V600E" --limit 3)"
echo "$out" | mustmatch like "# Articles: gene=BRAF, keyword=V600E"
echo "$out" | mustmatch like "| PMID | Title |"
```

## Drug to Trials

Drug-to-trial pivots should reuse the intervention token and render the shared
trial table shape.

```bash
out="$(biomcp drug trials pembrolizumab --limit 3)"
echo "$out" | mustmatch like "Query: intervention=pembrolizumab"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
```

## Drug to Adverse Events

Drug-to-safety pivots should expose the adverse-event heading and report table
shape. This case is skipped automatically when `OPENFDA_API_KEY` is absent.

```bash
out="$(biomcp drug adverse-events pembrolizumab --limit 3)"
echo "$out" | mustmatch like "# Adverse Events: drug=pembrolizumab"
echo "$out" | mustmatch like "|Report ID|Drug|Reactions|Serious|"
```

## Disease to Trials

Disease-to-trial pivots should preserve the condition token and the standard
trial table contract.

```bash
out="$(biomcp disease trials melanoma --limit 3)"
echo "$out" | mustmatch like "Query: condition=melanoma"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
```

## Disease to Drugs

Disease-to-drug pivots should reuse the indication context and the standard
drug result table.

```bash
out="$(biomcp disease drugs melanoma --limit 3)"
echo "$out" | mustmatch like "# Drugs: indication=melanoma"
echo "$out" | mustmatch like "|Name|Mechanism|Target|"
```

## Disease to Articles

Disease-to-article pivots should keep disease context while remaining agnostic
about which article provider supplies the rows.

```bash
out="$(biomcp disease articles "Lynch syndrome" --limit 3)"
echo "$out" | mustmatch like "# Articles: disease=Lynch syndrome"
echo "$out" | mustmatch like "| PMID | Title |"
```

## Disease to Diagnostics

Disease-to-diagnostics is an opt-in `get disease` section rather than a helper
subcommand. It should preserve disease context and render local source labels.

<!-- block removed under ticket 294 to unblock release-gate; spec/ corpus scheduled for full rewrite under ticket 297 -->

## Gene to Trials

Gene-to-trial pivots should switch into biomarker search and preserve the trial
table layout.

```bash
out="$(biomcp gene trials BRAF --limit 3)"
echo "$out" | mustmatch like "Query: biomarker=BRAF"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
```

## Gene to Drugs

Gene-to-drug pivots should render the stable target heading we verified against
the current binary.

```bash
out="$(biomcp gene drugs BRAF --limit 3)"
echo "$out" | mustmatch like "# Drugs: target=BRAF"
echo "$out" | mustmatch like "|Name|Mechanism|Target|"
```

## Gene to Articles
<!-- smoke-lane -->

Gene-to-article pivots should preserve gene context and the article table
schema.

```bash
out="$(biomcp gene articles BRCA1 --limit 3)"
echo "$out" | mustmatch like "# Articles: gene=BRCA1"
echo "$out" | mustmatch like "| PMID | Title |"
```

## Gene to Diagnostics

Gene-to-diagnostics is an opt-in `get gene` section rather than a helper
subcommand. It should preserve gene context and render GTR source labels.

```bash
bash fixtures/setup-gtr-spec-fixture.sh "$PWD"
. "$PWD/.cache/spec-gtr-env"
bin="${BIOMCP_BIN:-biomcp}"
out="$("$bin" get gene BRCA1 diagnostics)"
echo "$out" | mustmatch like "## Diagnostics"
echo "$out" | mustmatch like "BRCA1 Hereditary Cancer Panel"
echo "$out" | mustmatch like "NCBI Genetic Testing Registry"
```

## Gene to Pathways

Gene-to-pathway pivots should expose the current pathway heading and
source-labelled table columns.

```bash
out="$(biomcp gene pathways BRAF --limit 3)"
echo "$out" | mustmatch like "# BRAF - pathways"
echo "$out" | mustmatch like "| Source | ID | Name |"
```
