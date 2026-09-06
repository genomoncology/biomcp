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

## Typed EMA Search Identity

EMA search admits only typed fields from a MyChem hit that exactly identifies
the request. A systematic DrugBank synonym cannot leak its generic `acid`
token into unrelated EMA medicines, and every retained EU row explains its
match.

```bash
../../tools/biomcp-ci search drug eflornithine --region eu --limit 5 | mustmatch like '|Name|Active Substance|EMA Number|Status|Match|
|Eflornithine|other substance|EMEA/H/C/009001|Exact product|product_name: eflornithine (query)|
|Vaniqa|eflornithine|EMEA/H/C/000379|Authorised|active_substance: eflornithine (query)|'
../../tools/biomcp-ci search drug eflornithine --region eu --limit 5 | mustmatch not like 'Prasugrel Viatris'
../../tools/biomcp-ci search drug eflornithine --region all --limit 5 | mustmatch like '## EU (EMA)
product_name: eflornithine (query)'
../../tools/biomcp-ci --json search drug eflornithine --region eu --limit 1 | jq -e '.regions.eu as $eu | ($eu.pagination.total == 2) and $eu.pagination.has_more and ($eu.continuation_command | endswith("--offset 1")) and ($eu.results[0] | .match_kind == "product_name" and .matched_term == "eflornithine" and .source == "query")' | mustmatch 'true'
../../tools/biomcp-ci --json search drug eflornithine --region eu --limit 1 --offset 1 | jq -e '.regions.eu as $eu | ($eu.pagination.total == 2) and ($eu.pagination.has_more == false) and ($eu.results[0].name == "Vaniqa")' | mustmatch 'true'
../../tools/biomcp-ci --json search drug eflornithine --region eu --limit 1 --offset 2 | jq -e '.regions.eu as $eu | ($eu.pagination.total == 2) and ($eu.count == 0) and ($eu.pagination.has_more == false) and ($eu.continuation_command == null)' | mustmatch 'true'
../../tools/biomcp-ci --json search drug eflornithine --region eu --limit 1 --offset 99 | jq -e '.regions.eu as $eu | ($eu.pagination.total == 2) and ($eu.count == 0) and ($eu.pagination.has_more == false)' | mustmatch 'true'
```

Raw MCP executes the same CLI surface in readable and structured modes, while
the typed `search` schema remains intentionally narrower and still rejects a
drug entity.

```bash
python3 - <<'PY' | mustmatch like 'raw MCP EMA match facts agree'
import json, os, subprocess

proc = subprocess.Popen([os.environ["BIOMCP_BIN"], "serve"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, env=os.environ.copy())
def call(message):
    proc.stdin.write(json.dumps(message) + "\n")
    proc.stdin.flush()
    return json.loads(proc.stdout.readline())

call({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"spec","version":"1"}}})
proc.stdin.write(json.dumps({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}) + "\n")
proc.stdin.flush()
readable = call({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"biomcp","arguments":{"command":"biomcp search drug eflornithine --region eu --limit 1"}}})["result"]
structured = call({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"biomcp","arguments":{"command":"biomcp search drug eflornithine --region eu --limit 1","json":True}}})["result"]
tools = call({"jsonrpc":"2.0","id":4,"method":"tools/list","params":{}})["result"]["tools"]
assert readable.get("isError") is False and structured.get("isError") is False
assert "product_name: eflornithine (query)" in readable["content"][0]["text"]
payload = json.loads(structured["content"][0]["text"])
row = payload["regions"]["eu"]["results"][0]
assert (row["match_kind"], row["matched_term"], row["source"]) == ("product_name", "eflornithine", "query")
search_schema = next(tool["inputSchema"] for tool in tools if tool["name"] == "search")
assert '"const":"drug"' not in json.dumps(search_schema, separators=(",", ":"))
proc.terminate(); proc.wait(timeout=5)
print("raw MCP EMA match facts agree")
PY
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
(../../tools/biomcp-ci drug adverse-events osimertinib --type recall --count patient.reaction.reactionmeddrapt.exact 2>&1 || true) | mustmatch like '--type recall does not support: --count'
```

## Indication Structured Search

A structured indication miss is still informative. BioMCP should say that the
regulatory evidence is absent and point the user toward broader literature.

```bash
../../tools/biomcp-ci search drug --indication 'Marfan syndrome' --limit 3 | mustmatch like 'This absence is informative
biomcp search article -k "Marfan syndrome treatment" --type review --limit 5
Try: biomcp discover "Marfan syndrome"'
```

## FAERS Report-Share Semantics

Aggregate reaction percentages use matching FAERS reports as their denominator.
They are not incidence estimates and do not establish causality.

```bash
../../tools/biomcp-ci drug adverse-events pembrolizumab --limit 1 | mustmatch like '| Reaction | Count | Share of matching reports |
| MALIGNANT NEOPLASM PROGRESSION | 12016 | 100.0% |
not incidence
does not establish causality'
../../tools/biomcp-ci --json drug adverse-events pembrolizumab --limit 1 | jq -e '.summary.percentage_context == {"measure":"share_of_faers_reports","denominator":"all_matching_reports","denominator_count":12016,"is_incidence":false,"establishes_causality":false}'
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
grep -F 'GET /mychem/v1/query?q=trastuzumab&size=50&from=0&fields=' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'drugbank.synonyms%2Cchembl.molecule_chembl_id'
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
