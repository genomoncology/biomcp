# Drug and Safety Queries

Drug commands connect mechanism and target context with trial and adverse-event pivots. This file checks both core drug retrieval and OpenFDA-backed safety summaries. Assertions use durable headings and table columns instead of volatile report content.

| Section | Command focus | Why it matters |
|---|---|---|
| Drug search | `search drug pembrolizumab` | Confirms name-based lookup |
| Drug detail | `get drug pembrolizumab` | Confirms mechanism/target card |
| Targets section | `get drug ... targets` | Confirms progressive disclosure |
| Trial helper | `drug trials pembrolizumab` | Confirms intervention-based trial pivot |
| Adverse-event helper | `drug adverse-events pembrolizumab` | Confirms safety signal pivot |
| Adverse-event search | `search adverse-event -d ibuprofen` | Confirms direct safety search |

## Searching by Name

Name-first search is the most common route when reviewing a therapy in context. The card should provide a consistent heading and compact table schema.

```bash
out="$(biomcp search drug pembrolizumab --limit 3)"
echo "$out" | mustmatch like "# Drugs: pembrolizumab"
echo "$out" | mustmatch like "|Name|Mechanism|Target|"
```

## Getting Drug Details

`get drug` expands mechanism, targets, indications, and key metadata. We assert on the normalized heading and a stable metadata/section marker.

```bash
out="$(biomcp get drug pembrolizumab)"
echo "$out" | mustmatch like "# pembrolizumab"
echo "$out" | mustmatch like "DrugBank: DB09037"
echo "$out" | mustmatch like "## Targets"
```

## Drug Targets

Target-only expansion is useful when the workflow is gene-centric. This check ensures the section heading and expected target token are present.

```bash
out="$(biomcp get drug pembrolizumab targets)"
echo "$out" | mustmatch like "## Targets"
echo "$out" | mustmatch like "PDCD1"
```

## Drug Interactions With Public Label Text

The public MyChem payload does not reliably expose structured DrugBank interaction rows, so BioMCP should render OpenFDA label text when it exists instead of claiming no interactions are known.

```bash
out="$(biomcp get drug Warfarin interactions)"
echo "$out" | mustmatch like "## Interactions"
echo "$out" | mustmatch like "DRUG INTERACTIONS"
echo "$out" | mustmatch not like "No known drug-drug interactions found."
```

## Drug Interactions Truthful Fallback

When public label text is also unavailable, the interactions section must say so explicitly rather than implying the drug has no interactions.

```bash
out="$(biomcp get drug pembrolizumab interactions)"
echo "$out" | mustmatch like "## Interactions"
echo "$out" | mustmatch like "Interaction details not available from public sources."
echo "$out" | mustmatch not like "No known drug-drug interactions found."
```

## Drug to Trials

Intervention-based helper search should return the shared trial table layout. We also assert query echo to confirm the pivot preserved the drug token.

```bash
out="$(biomcp drug trials pembrolizumab --limit 3)"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
echo "$out" | mustmatch like "intervention=pembrolizumab"
```

## Drug to Adverse Events

This helper links a therapy directly to adverse-event reporting data. Assertions target the adverse-event heading and report table columns.

```bash
out="$(biomcp drug adverse-events pembrolizumab --limit 3)"
echo "$out" | mustmatch like "# Adverse Events: drug=pembrolizumab"
echo "$out" | mustmatch like "|Report ID|Drug|Reactions|Serious|"
```

## Adverse Event Search

Direct adverse-event search is useful for safety reconnaissance independent of drug metadata. We verify the heading and stable summary marker.

```bash
out="$(biomcp search adverse-event -d ibuprofen --limit 3)"
echo "$out" | mustmatch like "# Adverse Events: drug=ibuprofen"
echo "$out" | mustmatch like "Total reports (OpenFDA)"
```

## Brand Name Search

Brand-only MyChem hits should still render search rows with a usable canonical name. This regression protects the Keytruda brand-name bug where totals were non-zero but rows were empty.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search drug Keytruda --limit 5)"
echo "$out" | mustmatch like "# Drugs: Keytruda"
echo "$out" | mustmatch like "|Name|Mechanism|Target|"
echo "$out" | mustmatch like "pembrolizumab"
```
