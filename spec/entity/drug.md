# Drug Queries

Drug lookups have to bridge brand names, regulatory regions, and sparse evidence
without pretending those are the same question. These canaries keep the drug
surface focused on region truthfulness, canonical identity routing, and the new
structured DDInter interaction workflow before operators widen to safety or
literature.

## Card command discovery

Successful drug cards expose one bounded follow-up projection. JSON places the
flattened list in `_meta.next_commands`; Markdown categorizes the same surviving
commands under `More:`, `All:`, and `See also:`. The list is capped at ten and
orders recovery, up to three unloaded sections, a regional `all` command, then
related pivots. Default requests load only `targets`, explicit sections load
only their tokens, and WHO cards omit standalone safety and shortage commands.

```bash
../../tools/biomcp-ci --json get drug pembrolizumab | jq -e '._meta.next_commands | length <= 10 and .[0] == "biomcp get drug pembrolizumab approvals" and .[1] == "biomcp get drug pembrolizumab label" and .[2] == "biomcp get drug pembrolizumab regulatory --region us"' | mustmatch 'true'
../../tools/biomcp-ci get drug pembrolizumab | mustmatch like 'More:
  biomcp get drug pembrolizumab approvals
  biomcp get drug pembrolizumab label
  biomcp get drug pembrolizumab regulatory --region us

All:
  biomcp get drug pembrolizumab all --region us'
../../tools/biomcp-ci --json get drug pembrolizumab all | jq -e '((._meta.next_commands | any(. == "biomcp get drug pembrolizumab all --region us")) | not) and (._meta.next_commands | any(. == "biomcp get drug pembrolizumab approvals"))' | mustmatch 'true'
```

The provider fixture exercises the production renderers, rather than only the
pure owner. It keeps item order and resolved identities, compares the exact
ordered command projection in single and two-item CLI cards, and checks that
rendering discovery does not add provider requests.

```bash
python3 - <<'PY' | mustmatch like 'drug-card production projection passed'
import json
import os
import select
import shlex
import subprocess
import tempfile
import time
from urllib.parse import parse_qs, urlparse

binary = os.environ["BIOMCP_BIN"]
base_env = os.environ.copy()
log = base_env["BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG"]
identities = ["imatinib", "warfarin"]
resolved = ["imatinib mesylate", "warfarin"]
expected = {
    "imatinib": [
        "biomcp get drug \"imatinib mesylate\" targets",
        "biomcp get drug \"imatinib mesylate\" approvals",
        "biomcp get drug \"imatinib mesylate\" label",
        "biomcp get drug \"imatinib mesylate\" regulatory --region us",
        "biomcp get drug \"imatinib mesylate\" all --region us",
        "biomcp search article --drug \"imatinib mesylate\" --type review --limit 5",
        "biomcp drug trials \"imatinib mesylate\"",
        "biomcp drug adverse-events \"imatinib mesylate\"",
        "biomcp search pgx -d \"imatinib mesylate\"",
        "biomcp get gene ABL1",
    ],
    "warfarin": [
        "biomcp get drug warfarin targets",
        "biomcp get drug warfarin approvals",
        "biomcp get drug warfarin label",
        "biomcp get drug warfarin regulatory --region us",
        "biomcp get drug warfarin all --region us",
        "biomcp search article --drug warfarin --type review --limit 5",
        "biomcp drug trials warfarin",
        "biomcp drug adverse-events warfarin",
        "biomcp search pgx -d warfarin",
        "biomcp get gene VKORC1",
    ],
}

def run(*args, extra_env=None, check=True):
    child_env = base_env | {"BIOMCP_CACHE_DIR": tempfile.mkdtemp()}
    child_env.update(extra_env or {})
    return subprocess.run(
        [binary, *args], env=child_env, text=True, stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, check=check, timeout=60,
    )

def cli_json(*args, extra_env=None):
    return json.loads(run("--json", *args, extra_env=extra_env).stdout)

def reset_log():
    open(log, "w", encoding="utf-8").close()

def request_count():
    return sum(bool(line.strip()) for line in open(log, encoding="utf-8"))

def wait_for_count(expected):
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline and request_count() < expected:
        time.sleep(0.01)
    return request_count()

def guidance_commands(markdown):
    commands = []
    for line in markdown.splitlines():
        line = line.strip()
        if line.startswith("biomcp "):
            commands.append(line.split("   -", 1)[0])
        elif line.startswith("Retry: `"):
            commands.append(line.removeprefix("Retry: `").split("`", 1)[0])
    return commands

reset_log()
single_json = cli_json("get", "drug", identities[0])
assert single_json["name"] == resolved[0]
assert single_json["_meta"]["next_commands"] == expected[identities[0]]
single_json_requests = wait_for_count(5)
assert single_json_requests >= 5
reset_log()
single_markdown = run("get", "drug", identities[0]).stdout
assert guidance_commands(single_markdown) == expected[identities[0]]
assert wait_for_count(5) == single_json_requests

multi_expected = [
    "biomcp get drug \"imatinib mesylate\" targets",
    "biomcp get drug \"imatinib mesylate\" approvals",
    "biomcp get drug \"imatinib mesylate\" regulatory --region us",
    "biomcp get drug \"imatinib mesylate\" safety --region us",
    "biomcp get drug \"imatinib mesylate\" all --region us",
    "biomcp search article --drug \"imatinib mesylate\" --type review --limit 5",
    "biomcp drug trials \"imatinib mesylate\"",
    "biomcp drug adverse-events \"imatinib mesylate\"",
    "biomcp search pgx -d \"imatinib mesylate\"",
    "biomcp get gene ABL1",
]
reset_log()
multi_json = cli_json("get", "drug", "imatinib", "label", "targets")
assert multi_json["_meta"]["next_commands"] == multi_expected
multi_json_requests = wait_for_count(4)
assert multi_json_requests >= 4
reset_log()
multi_markdown = run("get", "drug", "imatinib", "label", "targets").stdout
assert guidance_commands(multi_markdown) == multi_expected
assert wait_for_count(4) == multi_json_requests

reset_log()
batch_json = cli_json("batch", "drug", ",".join(identities))
assert batch_json["summary"] == {"total": 2, "succeeded": 2, "failed": 0}
assert [item["input"] for item in batch_json["items"]] == identities
assert [item["result"]["name"] for item in batch_json["items"]] == resolved
assert [item["result"]["_meta"]["next_commands"] for item in batch_json["items"]] == [expected[name] for name in identities]
batch_json_requests = wait_for_count(10)
assert batch_json_requests >= 10
reset_log()
batch_markdown = run("batch", "drug", ",".join(identities)).stdout
assert batch_markdown.index("## imatinib — ok") < batch_markdown.index("## warfarin — ok")
for identity in identities:
    start = batch_markdown.index("## " + identity + " — ok")
    end = batch_markdown.find("\n\n---", start)
    item = batch_markdown[start:] if end < 0 else batch_markdown[start:end]
    assert guidance_commands(item) == expected[identity]
assert wait_for_count(10) == batch_json_requests

def execute_emitted(command, extra_env=None):
    argv = shlex.split(command)
    assert argv[0] == "biomcp"
    result = run(*argv[1:], extra_env=extra_env, check=False)
    assert result.returncode == 0, (command, result.stderr)

execution_card = cli_json("get", "drug", "pembrolizumab")
for command in execution_card["_meta"]["next_commands"]:
    if command.endswith((" approvals", " all --region us")) or "drug trials" in command or "adverse-events" in command:
        execute_emitted(command)

with tempfile.NamedTemporaryFile() as missing_ema:
    recovery_env = {"BIOMCP_EMA_DIR": missing_ema.name}
    recovery = cli_json("get", "drug", "pembrolizumab", "safety", "--region", "eu", extra_env=recovery_env)
    assert recovery["section_outcomes"]["safety"]["outcome"] == "unavailable"
    assert recovery["_meta"]["next_commands"][0] == "biomcp get drug pembrolizumab safety --region eu"
    execute_emitted(recovery["_meta"]["next_commands"][0], recovery_env)
    recovery_markdown = run("get", "drug", "pembrolizumab", "safety", "--region", "eu", extra_env=recovery_env).stdout
    assert recovery_markdown.count("Retry: `biomcp get drug pembrolizumab safety --region eu`") == 1

with tempfile.NamedTemporaryFile(delete=False) as sentinel:
    sentinel_path = sentinel.name
os.unlink(sentinel_path)
hostile = "ticket1151 hostile 'x' \"q\" \\ $x `rm` ; & $(touch " + sentinel_path + ") path"
reset_log()
hostile_card = cli_json("get", "drug", hostile)
assert hostile_card["name"] == hostile
hostile_command = next(
    command for command in hostile_card["_meta"]["next_commands"]
    if command.endswith(" approvals")
)
reset_log()
shell_env = base_env | {"PATH": os.path.dirname(binary) + ":" + base_env.get("PATH", ""), "BIOMCP_CACHE_DIR": tempfile.mkdtemp()}
hostile_result = subprocess.run(hostile_command, shell=True, executable="/bin/bash", env=shell_env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=60)
assert hostile_result.returncode == 0
assert not os.path.exists(sentinel_path)
mychem = [line for line in open(log, encoding="utf-8") if line.startswith("GET /mychem/")]
assert mychem
for line in mychem:
    location = line.split(" ", 1)[1].split(" ", 1)[0]
    assert parse_qs(urlparse(location).query)["q"][0] == hostile

def rpc(proc, message):
    proc.stdin.write(json.dumps(message) + "\n")
    proc.stdin.flush()
    ready, _, _ = select.select([proc.stdout], [], [], 60)
    assert ready, "MCP response timeout"
    return json.loads(proc.stdout.readline())

server_env = base_env | {"BIOMCP_CACHE_DIR": tempfile.mkdtemp()}
server = subprocess.Popen(
    [binary, "serve"], env=server_env, stdin=subprocess.PIPE,
    stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True,
)
try:
    rpc(server, {"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{"protocolVersion":"2025-03-26", "capabilities":{}, "clientInfo":{"name":"spec", "version":"1"}}})
    server.stdin.write(json.dumps({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}) + "\n")
    server.stdin.flush()
    request_id = 2
    def mcp(tool, arguments):
        nonlocal_request_id[0] += 1
        result = rpc(server, {"jsonrpc":"2.0", "id":nonlocal_request_id[0], "method":"tools/call", "params":{"name":tool, "arguments":arguments}})["result"]
        assert result.get("isError") is False
        return result["content"][0]["text"]
    nonlocal_request_id = [1]
    for json_output in (False, True):
        cli_text = run(*((["--json"] if json_output else []) + ["get", "drug", "imatinib"])).stdout
        args = {"command":"biomcp get drug imatinib", "json":json_output}
        assert mcp("biomcp", args).rstrip("\n") == cli_text.rstrip("\n")
        typed = {"entity":"drug", "id":"imatinib", "sections":[], "json":json_output}
        assert mcp("get", typed).rstrip("\n") == cli_text.rstrip("\n")
    for json_output in (False, True):
        cli_args = (["--json"] if json_output else []) + ["batch", "drug", "imatinib,warfarin"]
        cli_text = run(*cli_args).stdout
        raw_args = {"command":"biomcp batch drug imatinib,warfarin", "json":json_output}
        assert mcp("biomcp", raw_args).rstrip("\n") == cli_text.rstrip("\n")
    tools = rpc(server, {"jsonrpc":"2.0", "id":99, "method":"tools/list", "params":{}})["result"]["tools"]
    assert [tool["name"] for tool in tools] == ["biomcp", "search", "get", "variant_normalize_car", "variant_erepo", "gene_cspec", "variant_articles"]
    get_schema = next(tool["inputSchema"] for tool in tools if tool["name"] == "get")
    drug_branch = next(branch for branch in get_schema["oneOf"] if branch["properties"]["entity"]["const"] == "drug")
    assert drug_branch["properties"]["sections"]["items"]["enum"] == ["label", "regulatory", "safety", "shortage", "targets", "indications", "interactions", "civic", "approvals", "all"]
finally:
    server.terminate()
    server.wait(timeout=5)

print("drug-card production projection passed")
PY
```

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
../../tools/biomcp-ci --json search drug trastuzumab --region all --limit 1 | jq -e '.regions as $r | (($r | keys) == ["eu","us","who"]) and ($r.eu.results[0] | .name == "Herceptin" and .match_kind == "active_substance" and .matched_term == "trastuzumab" and .source == "query") and ($r.us.results[0] | (keys == ["match_kind","mechanism","name"]) and .name == "trastuzumab" and .match_kind == "active_substance") and ($r.who.results[0] | (keys == ["applicant","dosage_form","inn","listing_basis","match_kind","prequalification_date","presentation","product_type","therapeutic_area","who_reference_number"]) and .inn == "Trastuzumab" and .who_reference_number == "BT-ON001" and .match_kind == "active_substance")' | mustmatch 'true'
../../tools/biomcp-ci --json search drug eflornithine --region eu --limit 1 | jq -e '.regions.eu as $eu | ($eu.pagination.total == 3) and $eu.pagination.has_more and ($eu.continuation_command | endswith("--offset 1")) and ($eu.results[0] | .match_kind == "product_name" and .matched_term == "eflornithine" and .source == "query")' | mustmatch 'true'
../../tools/biomcp-ci --json search drug eflornithine --region eu --limit 1 --offset 1 | jq -e '.regions.eu as $eu | ($eu.pagination.total == 3) and ($eu.pagination.has_more == true) and ($eu.continuation_command | endswith("--offset 2")) and ($eu.results[0].name == "Vaniqa")' | mustmatch 'true'
../../tools/biomcp-ci --json search drug eflornithine --region eu --limit 1 --offset 2 | jq -e '.regions.eu as $eu | ($eu.pagination.total == 3) and ($eu.pagination.has_more == false) and ($eu.continuation_command == null) and ($eu.results[0].name == "Eflornithine cream") and ($eu.results[0].match_kind == "broad_text")' | mustmatch 'true'
../../tools/biomcp-ci --json search drug eflornithine --region eu --limit 1 --offset 3 | jq -e '.regions.eu as $eu | ($eu.pagination.total == 3) and ($eu.count == 0) and ($eu.pagination.has_more == false) and ($eu.continuation_command == null)' | mustmatch 'true'
../../tools/biomcp-ci --json search drug eflornithine --region eu --limit 1 --offset 99 | jq -e '.regions.eu as $eu | ($eu.pagination.total == 3) and ($eu.count == 0) and ($eu.pagination.has_more == false) and ($eu.continuation_command == null)' | mustmatch 'true'
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
