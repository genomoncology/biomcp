# Cross-Entity Pivot Guide Contracts

This spec protects the public workflow guide at
`docs/how-to/cross-entity-pivots.md`. It covers the guide copy newcomers rely
on, the entry points that should lead them there, and the runnable pivot
commands that the guide teaches.

| Surface | Representative checks | Why it matters |
|---|---|---|
| Guide page | Decision section and family headings | Protects the bounded docs slice |
| Entry points | README, docs home, getting started, quick reference | Keeps newcomer routes connected |
| Variant pivots | `variant trials`, `variant articles` | Mutation-first investigation flow |
| Drug pivots | `drug trials`, `drug adverse-events` | Therapy-to-trial and therapy-to-safety flow |
| Disease pivots | `disease trials`, `disease drugs`, `disease articles` | Diagnosis-centered pivots |
| Gene pivots | `gene trials`, `gene drugs`, `gene articles`, `gene pathways` | Canonical biomarker pivots |

## Guide page

The guide should exist as one dedicated how-to page and explain the helper
boundary before it starts the family walkthroughs. These checks assert the key
headings and command examples the ticket requires.

```bash
root="$(git rev-parse --show-toplevel)"
out="$(cat "$root/docs/how-to/cross-entity-pivots.md")"
echo "$out" | mustmatch like "# How To: Cross-Entity Pivots"
echo "$out" | mustmatch like "## When to use a pivot helper vs. a fresh search"
echo "$out" | mustmatch like "already have a specific entity identifier or label"
echo "$out" | mustmatch like "--status"
echo "$out" | mustmatch like "--phase"
echo "$out" | mustmatch like "--since"
echo "$out" | mustmatch like "## Variant pivots"
echo "$out" | mustmatch like "## Drug pivots"
echo "$out" | mustmatch like "## Disease pivots"
echo "$out" | mustmatch like "## Gene pivots"
echo "$out" | mustmatch like "biomcp variant trials \"BRAF V600E\" --limit 5"
echo "$out" | mustmatch like "biomcp drug adverse-events pembrolizumab --limit 5"
echo "$out" | mustmatch like "biomcp disease articles \"Lynch syndrome\" --limit 5"
echo "$out" | mustmatch like "biomcp gene pathways BRAF --limit 5"
```

## Docs navigation

The docs site nav should list the new guide under the existing How-To section
so the built site exposes it as a first-class task-oriented page.

```bash
root="$(git rev-parse --show-toplevel)"
out="$(cat "$root/mkdocs.yml")"
echo "$out" | mustmatch like "  - How-To:"
echo "$out" | mustmatch like "      - Cross-Entity Pivots: how-to/cross-entity-pivots.md"
```

## README entry point

The repository landing page already teaches helper syntax. It should also point
readers to the dedicated guide rather than leaving the examples unexplained.

```bash
root="$(git rev-parse --show-toplevel)"
out="$(cat "$root/README.md")"
echo "$out" | mustmatch like "See the [cross-entity pivot guide](docs/how-to/cross-entity-pivots.md)"
```

## Docs home entry point

The docs home page introduces cross-entity helpers early, so it should direct
newcomers to the guide from that section.

```bash
root="$(git rev-parse --show-toplevel)"
out="$(cat "$root/docs/index.md")"
echo "$out" | mustmatch like "[cross-entity pivot guide](how-to/cross-entity-pivots.md)"
```

## First query entry point

The getting-started walkthrough should send users from their first successful
query to the dedicated pivot workflow guide.

```bash
root="$(git rev-parse --show-toplevel)"
out="$(cat "$root/docs/getting-started/first-query.md")"
echo "$out" | mustmatch like "[cross-entity pivot guide](../how-to/cross-entity-pivots.md)"
```

## Quick reference entry point

The quick reference page is a common lookup surface for command grammar. Its
related references list should include the dedicated pivot guide.

```bash
root="$(git rev-parse --show-toplevel)"
out="$(cat "$root/docs/reference/quick-reference.md")"
echo "$out" | mustmatch like "[Cross-Entity Pivot Guide](../how-to/cross-entity-pivots.md)"
```

## Variant pivots

Variant helpers should preserve the mutation context when crossing into trials
or articles. The docs only promise stable headings and table shapes.

```bash
out="$(biomcp variant trials "BRAF V600E" --limit 3)"
echo "$out" | mustmatch like "Query: mutation=BRAF V600E"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
```

Variant-to-article pivots should keep gene and keyword context without
promising provider-specific subsections or counts.

```bash
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

Gene-to-article pivots should preserve gene context and the article table
schema.

```bash
out="$(biomcp gene articles BRCA1 --limit 3)"
echo "$out" | mustmatch like "# Articles: gene=BRCA1"
echo "$out" | mustmatch like "| PMID | Title |"
```

## Gene to Pathways

Gene-to-pathway pivots should expose the current pathway heading and
source-labelled table columns.

```bash
out="$(biomcp gene pathways BRAF --limit 3)"
echo "$out" | mustmatch like "# BRAF - pathways"
echo "$out" | mustmatch like "| Source | ID | Name |"
```
