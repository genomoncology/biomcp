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
../../tools/biomcp-ci search drug trastuzumab --limit 3 | mustmatch like '## US (MyChem.info / OpenFDA)
## EU (EMA)
## WHO (WHO Prequalification)'
../../tools/biomcp-ci search drug trastuzumab --limit 3 | mustmatch '/\|Trastuzumab\|Biotherapeutic Product\|[^|]+\|[^|]+\|[^|]+\|BT-ON[0-9]+\|/'
```

## Brand-Name Bridge

Brand-name `get` requests should land on the canonical generic identity, not a
brand-local card that keeps all downstream commands on the alias spelling.

```bash
../../tools/biomcp-ci get drug Keytruda | mustmatch like '# pembrolizumab
biomcp drug trials pembrolizumab'
```

## Research-Code Bridge

Quarantined from routine `make spec-pr` by ticket 382. The former live
`MK-3475` assertions expected the paper/sponsor code to canonicalize to
`pembrolizumab` and keep next commands on the INN, but current runtime can emit
an `mk-3475` card and paper-code trial pivot instead. That alias behavior is a
drug canonicalization question, not a routine PR-gate blocker.

Keep this heading as the restoration anchor. Bring the behavior back only as a
fixture-backed drug alias/canonicalization request contract, or as an explicit
release/live-smoke canary after the ticket 371 request-contract reset reaches
drug alias surfaces.

## Ambiguous Research-Code Fallback

Quarantined from routine `make spec-pr` by ticket 380. The former live
`MK-7684` assertion depended on ambiguous upstream drug discovery behavior and
blocked unrelated March work when the runtime returned not-found search guidance
instead of alias-disambiguation text.

Keep this heading as the restoration anchor. Bring the behavior back only as a
fixture-backed alias/disambiguation contract or as an explicit release/live-smoke
canary after the ticket 371 request-contract reset reaches drug/alias surfaces.

## Adverse-Event Aggregate Filter Surface

The `drug adverse-events` helper must accept the FAERS filters it advertises,
especially `--count`, instead of failing in clap before the adverse-event path
can render aggregate rankings.

```bash
../../tools/biomcp-ci drug adverse-events --help | mustmatch like '--count <COUNT>'
../../tools/biomcp-ci drug adverse-events --help | mustmatch like '--reaction <REACTION>'
../../tools/biomcp-ci drug adverse-events --help | mustmatch like 'osimertinib --count patient.reaction.reactionmeddrapt.exact'
(../../tools/biomcp-ci drug adverse-events osimertinib --type recall --count patient.reaction.reactionmeddrapt.exact 2>&1 || true) | mustmatch like '--count are only valid for --type faers'
```

## Indication Structured Search

A structured indication miss is still informative. BioMCP should say that the
regulatory evidence is absent and point the user toward broader literature.

```bash
../../tools/biomcp-ci search drug --indication 'Marfan syndrome' --limit 3 | mustmatch like 'This absence is informative
biomcp search article -k "Marfan syndrome treatment" --type review --limit 5
Try: biomcp discover "Marfan syndrome"'
```

## WHO Regulatory Detail

WHO prequalification should stay readable as a regional table with the stable
columns operators need for procurement and regulatory review.

```bash
../../tools/biomcp-ci get drug trastuzumab regulatory --region who | mustmatch like '## Regulatory (WHO Prequalification)
| WHO ID | Type | Presentation / INN |
Samsung Bioepis NL B.V.'
```

## US Regulatory Detail

The U.S. overlay must decode the Drugs@FDA response and preserve its source,
application, product, and submission fields.

```bash
../../tools/biomcp-ci get drug imatinib regulatory --region us | mustmatch like '## Regulatory (US - Drugs@FDA)
### NDA021588
- Sponsor: NOVARTIS
| GLEEVEC | TABLET | oral | Prescription |
| ORIG | 1 | AP | 2003-04-18 |'
```

## Observed Provider Requests

The routine fixture records the requests emitted by production clients. These
checks keep method, route, query, paging, and requested field contracts visible.

```bash
grep -F 'GET /mychem/v1/query?q=trastuzumab&size=6&from=0&fields=' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'chembl.molecule_chembl_id'
grep -F 'GET /openfda/drug/drugsfda.json?search=' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '&limit=8&skip=0'
```

## Targets & Trial Pivots

Regional regulatory detail should not crowd out targetability or the related
trial/adverse-event pivots that a clinician uses from the same card.

```bash
../../tools/biomcp-ci get drug pembrolizumab targets regulatory --region eu | mustmatch like '## Regulatory (EU - EMA)
## Targets (ChEMBL / Open Targets)
biomcp drug trials pembrolizumab'
../../tools/biomcp-ci get drug pembrolizumab targets regulatory --region eu | mustmatch '/PDCD1\nMore:/'
grep -F 'GET /chembl/mechanism.json?molecule_chembl_id=CHEMBL3137343&limit=15' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'CHEMBL3137343'
grep -F 'POST /opentargets/api/v4/graphql' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"chemblId":"CHEMBL3137343"'
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
