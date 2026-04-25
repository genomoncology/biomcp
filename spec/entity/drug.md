# Drug Queries

Drug lookups have to bridge brand names, regulatory regions, and sparse evidence
without pretending those are the same question. These canaries keep the drug
surface focused on region truthfulness, canonical identity routing, and the new
structured DDInter interaction workflow before operators widen to safety or
literature.

## Multi-Region Search

Plain-name search should still show the same drug family across the U.S., EU,
and WHO views so operators can compare regulatory coverage in one place.

```bash
out="$(../../tools/biomcp-ci search drug trastuzumab --limit 3)"
echo "$out" | mustmatch like "## US (MyChem.info / OpenFDA)"
echo "$out" | mustmatch like "## EU (EMA)"
echo "$out" | mustmatch like "## WHO (WHO Prequalification)"
```

## Brand-Name Bridge

Brand-name `get` requests should land on the canonical generic identity, not a
brand-local card that keeps all downstream commands on the alias spelling.

```bash
out="$(../../tools/biomcp-ci get drug Keytruda)"
echo "$out" | mustmatch like "# pembrolizumab"
echo "$out" | mustmatch like "biomcp drug trials pembrolizumab"
```

## Structured Drug Interactions

When the question is explicitly about drug-drug interactions, the helper should
surface a dedicated DDInter-backed report instead of asking the operator to
infer partner classes from a generic drug card.

```bash
out="$(../../tools/biomcp-ci drug interactions warfarin)"
echo "$out" | mustmatch like "# warfarin"
echo "$out" | mustmatch like "## Interacting Drug Classes"
echo "$out" | mustmatch like "anti-infectives"
echo "$out" | mustmatch like "antiplatelets"
echo "$out" | mustmatch like "| statins |"
```

## Oncology Interaction Class Rollups

The same helper should stay useful for oncology drugs, where class-level
grouping is often more actionable than a long flat list of partner rows.

```bash
out="$(../../tools/biomcp-ci drug interactions imatinib)"
echo "$out" | mustmatch like "# imatinib"
echo "$out" | mustmatch like "## Interacting Drug Classes"
echo "$out" | mustmatch like "| CYP3A4 |"
```

## Indication Structured Search

A structured indication miss is still informative. BioMCP should say that the
regulatory evidence is absent and point the user toward broader literature.

```bash
out="$(../../tools/biomcp-ci search drug --indication 'Marfan syndrome' --limit 3)"
echo "$out" | mustmatch like "This absence is informative"
echo "$out" | mustmatch like 'biomcp search article -k "Marfan syndrome treatment" --type review --limit 5'
echo "$out" | mustmatch like 'Try: biomcp discover "Marfan syndrome"'
```

## WHO Regulatory Detail

WHO prequalification should stay readable as a regional table with the stable
columns operators need for procurement and regulatory review.

```bash
out="$(../../tools/biomcp-ci get drug trastuzumab regulatory --region who)"
echo "$out" | mustmatch like "## Regulatory (WHO Prequalification)"
echo "$out" | mustmatch like "| WHO ID | Type | Presentation / INN |"
echo "$out" | mustmatch like "Samsung Bioepis NL B.V."
```

## Section Parity for Interaction Detail

`get drug <name> interactions` should render the same DDInter-backed interaction
contract as the helper instead of falling back to a separate low-fidelity
interaction section.

```bash
out="$(../../tools/biomcp-ci get drug warfarin interactions)"
echo "$out" | mustmatch like "## Interactions (DDInter)"
echo "$out" | mustmatch like "## Interacting Drug Classes"
echo "$out" | mustmatch like "anti-infectives"
```

## Targets & Trial Pivots

Regional regulatory detail should not crowd out targetability or the related
trial/adverse-event pivots that a clinician uses from the same card.

```bash
out="$(../../tools/biomcp-ci get drug pembrolizumab targets regulatory --region eu)"
echo "$out" | mustmatch like "## Regulatory (EU - EMA)"
echo "$out" | mustmatch like "## Targets (ChEMBL / Open Targets)"
echo "$out" | mustmatch like "biomcp drug trials pembrolizumab"
```

## Truthful Source-Empty Interaction State

DDInter empty states should be phrased as source empties. BioMCP must never
turn a missing DDInter row into a claim that the anchor drug has no clinical
interactions.

```bash
out="$(../../tools/biomcp-ci drug interactions daraxonrasib)"
echo "$out" | mustmatch like "current DDInter download bundle has no matching rows"
echo "$out" | mustmatch not like "no clinical interactions"
```
