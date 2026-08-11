# Drug Target and Interaction Live Queries

These remaining upstream-dependent checks move back into the routine drug page
when ticket 0905 supplies receipted ChEMBL and DDInter contracts.

## Targets & Trial Pivots

Regional regulatory detail should not crowd out targetability or the related
trial/adverse-event pivots that a clinician uses from the same card.

```bash
../../tools/biomcp-ci get drug pembrolizumab targets regulatory --region eu | mustmatch like '## Regulatory (EU - EMA)
## Targets (ChEMBL / Open Targets)
biomcp drug trials pembrolizumab'
../../tools/biomcp-ci get drug pembrolizumab targets regulatory --region eu | mustmatch '/PDCD1\nMore:/'
```

## Truthful Source-Empty Interaction State

DDInter empty states should be phrased as source empties. BioMCP must never
turn a missing DDInter row into a claim that the anchor drug has no clinical
interactions.

```bash
../../tools/biomcp-ci drug interactions daraxonrasib | mustmatch like 'current DDInter download bundle has no matching rows'
../../tools/biomcp-ci drug interactions daraxonrasib | mustmatch not like 'no clinical interactions'
```

Uncovered drugs should also carry a structured coverage status so agents can
branch on a source-coverage miss instead of treating an empty table as safety
evidence.

```bash
../../tools/biomcp-ci --json drug interactions dabigatran | mustmatch like '"coverage_status": "not_in_ddinter_coverage"'
../../tools/biomcp-ci drug interactions dabigatran | mustmatch like 'current DDInter download bundle has no matching rows
not_in_ddinter_coverage
source coverage miss'
```
