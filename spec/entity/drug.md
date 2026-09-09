# Drug Queries

Drug lookups have to bridge brand names, regulatory regions, and sparse evidence
without pretending those are the same question. These canaries keep the drug
surface focused on region truthfulness, canonical identity routing, and the new
structured DDInter interaction workflow before operators widen to safety or
literature.

## Card follow-up projection

Every successful drug card uses one projection for Markdown guidance and JSON
`_meta.next_commands`. The list is capped at ten and ordered as loaded-section
retries, up to three not-loaded section commands, an optional regional `all`
command, then related pivots. Default cards implicitly load only `targets`;
explicit section tokens load exactly those sections. `all` expands to
`label`, `regulatory`, `safety`, `shortage`, `targets`, `indications`,
`interactions`, and `civic`, leaving legacy `approvals` discoverable. Regional
section and aggregate commands carry the canonical effective region, and WHO
cards omit standalone safety and shortage commands.

```bash
../../tools/biomcp-ci --json get drug pembrolizumab | jq -e '._meta.next_commands | length <= 10 and .[0] == "biomcp get drug pembrolizumab approvals" and .[1] == "biomcp get drug pembrolizumab label" and .[2] == "biomcp get drug pembrolizumab regulatory --region us"' | mustmatch 'true'
../../tools/biomcp-ci get drug pembrolizumab | mustmatch like 'More:'
../../tools/biomcp-ci get drug pembrolizumab all | mustmatch not like 'All:'
../../tools/biomcp-ci --json get drug pembrolizumab all | jq -e '._meta.next_commands[0] == "biomcp get drug pembrolizumab approvals" and (._meta.next_commands | any(. == "biomcp get drug pembrolizumab all --region us") | not)' | mustmatch 'true'
```

## Drug-card projection production matrix

The existing provider-contract fixture exercises the shipped CLI and both MCP
entry points with the same resolved identities. This matrix keeps the exact
single-card, two-item batch, recovery, parser, request-count, and tool-surface
contracts executable without adding a second fixture family.

```bash
python3 - <<'PY' | mustmatch like 'drug-card projection production matrix passed'
import json
import os
import select
import subprocess
import tempfile

binary = os.environ["BIOMCP_BIN"]
env = os.environ.copy()
log = env["BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG"]
ddinter = tempfile.TemporaryDirectory()
env["BIOMCP_DDINTER_DIR"] = ddinter.name

def cli(*args, extra_env=None):
    with tempfile.TemporaryDirectory() as cache:
        child_env = env | {"BIOMCP_CACHE_DIR": cache} | (extra_env or {})
        result = subprocess.run(
            [binary, *args], cwd=os.environ["PWD"], env=child_env,
            check=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
            text=True, timeout=60,
        )
        return result.stdout

def shell(command, extra_env=None):
    with tempfile.TemporaryDirectory() as cache:
        child_env = env | {"BIOMCP_CACHE_DIR": cache, "PATH": os.path.dirname(binary) + ":" + env.get("PATH", "")} | (extra_env or {})
        return subprocess.run(
            command, shell=True, executable="/bin/bash", cwd=os.environ["PWD"],
            env=child_env, check=True, stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL, timeout=60,
        )

def json_card(*args, extra_env=None):
    return json.loads(cli("--json", *args, extra_env=extra_env))

def guidance(markdown):
    start = markdown.index("More:\n")
    end = markdown.find("\n\n[", start)
    return markdown[start:] if end < 0 else markdown[start:end] + "\n"

default = [
    "biomcp get drug pembrolizumab approvals",
    "biomcp get drug pembrolizumab label",
    "biomcp get drug pembrolizumab regulatory --region us",
    "biomcp get drug pembrolizumab all --region us",
    "biomcp search article --drug pembrolizumab --type review --limit 5",
    "biomcp drug trials pembrolizumab",
    "biomcp drug adverse-events pembrolizumab",
    "biomcp search pgx -d pembrolizumab",
    "biomcp get gene PDCD1",
]
all_commands = [
    "biomcp get drug pembrolizumab interactions",
    "biomcp get drug pembrolizumab approvals",
    "biomcp drug trials pembrolizumab",
    "biomcp drug adverse-events pembrolizumab",
    "biomcp search pgx -d pembrolizumab",
    "biomcp get gene PDCD1",
]
multi = [
    "biomcp get drug pembrolizumab approvals",
    "biomcp get drug pembrolizumab regulatory --region us",
    "biomcp get drug pembrolizumab safety --region us",
    "biomcp get drug pembrolizumab all --region us",
    "biomcp search article --drug pembrolizumab --type review --limit 5",
    "biomcp drug trials pembrolizumab",
    "biomcp drug adverse-events pembrolizumab",
    "biomcp search pgx -d pembrolizumab",
    "biomcp get gene PDCD1",
]
eu = [
    "biomcp get drug pembrolizumab safety --region eu",
    "biomcp get drug pembrolizumab approvals",
    "biomcp get drug pembrolizumab label",
    "biomcp get drug pembrolizumab regulatory --region eu",
    "biomcp get drug pembrolizumab all --region eu",
    "biomcp search article --drug pembrolizumab --type review --limit 5",
    "biomcp drug trials pembrolizumab",
    "biomcp drug adverse-events pembrolizumab",
    "biomcp search pgx -d pembrolizumab",
    "biomcp get gene PDCD1",
]
assert json_card("get", "drug", "pembrolizumab")["_meta"]["next_commands"] == default
assert json_card("get", "drug", "pembrolizumab", "all")["_meta"]["next_commands"] == all_commands
assert json_card("get", "drug", "pembrolizumab", "label", "targets")["_meta"]["next_commands"] == multi

default_guidance = """More:
  biomcp get drug pembrolizumab approvals   - Drugs@FDA approval history
  biomcp get drug pembrolizumab label   - approved-indication and FDA label detail beyond the base card
  biomcp get drug pembrolizumab regulatory --region us   - approval and supplement history; use only if the base card lacks approval context

All:
  biomcp get drug pembrolizumab all --region us
See also:
  biomcp search article --drug pembrolizumab --type review --limit 5   - supplement sparse structured data with review literature for indication context
  biomcp drug trials pembrolizumab
  biomcp drug adverse-events pembrolizumab   - inspect safety reports and adverse-event signal
  biomcp search pgx -d pembrolizumab   - pharmacogenomics interactions
  biomcp get gene PDCD1
"""
assert guidance(cli("get", "drug", "pembrolizumab")) == default_guidance
all_guidance = """More:
  biomcp get drug pembrolizumab approvals   - Drugs@FDA approval history
See also:
  biomcp drug trials pembrolizumab
  biomcp drug adverse-events pembrolizumab   - inspect safety reports and adverse-event signal
  biomcp search pgx -d pembrolizumab   - pharmacogenomics interactions
  biomcp get gene PDCD1
"""
assert guidance(cli("get", "drug", "pembrolizumab", "all")) == all_guidance
multi_guidance = """More:
  biomcp get drug pembrolizumab approvals   - Drugs@FDA approval history
  biomcp get drug pembrolizumab regulatory --region us   - approval and supplement history; use only if the base card lacks approval context
  biomcp get drug pembrolizumab safety --region us   - regulatory safety detail; use `biomcp drug adverse-events <name>` first when you want post-marketing signal

All:
  biomcp get drug pembrolizumab all --region us
See also:
  biomcp search article --drug pembrolizumab --type review --limit 5   - supplement sparse structured data with review literature for indication context
  biomcp drug trials pembrolizumab
  biomcp drug adverse-events pembrolizumab   - inspect safety reports and adverse-event signal
  biomcp search pgx -d pembrolizumab   - pharmacogenomics interactions
  biomcp get gene PDCD1
"""
assert guidance(cli("get", "drug", "pembrolizumab", "label", "targets")) == multi_guidance

default_trast = [command.replace("pembrolizumab", "trastuzumab") for command in default[:-1]]
all_trast = [
    "biomcp get drug trastuzumab interactions",
    "biomcp get drug trastuzumab approvals",
    "biomcp search article --drug trastuzumab --type review --limit 5",
    "biomcp drug trials trastuzumab",
    "biomcp drug adverse-events trastuzumab",
    "biomcp search pgx -d trastuzumab",
]
multi_trast = [command.replace("pembrolizumab", "trastuzumab") for command in multi[:-1]]

def expected_guidance(identity, mode, commands):
    labels = {
        "approvals": "   - Drugs@FDA approval history",
        "label": "   - approved-indication and FDA label detail beyond the base card",
        "regulatory": "   - approval and supplement history; use only if the base card lacks approval context",
        "safety": "   - regulatory safety detail; use `biomcp drug adverse-events <name>` first when you want post-marketing signal",
    }
    lines = ["More:"]
    if mode == "default":
        more = commands[:3]
    elif mode == "multi":
        more = commands[:3]
    else:
        more = [commands[1]]
    for command in more:
        section = next((name for name in labels if " drug " + identity + " " + name in command), "")
        lines.append("  " + command + labels.get(section, ""))
    if mode != "all":
        lines.extend(["", "All:", "  " + next(command for command in commands if " all " in command)])
    lines.append("See also:")
    related = commands[(4 if mode != "all" else 2):]
    lines.extend("  " + command + ("   - supplement sparse structured data with review literature for indication context" if "search article" in command else "   - inspect safety reports and adverse-event signal" if "adverse-events" in command else "   - pharmacogenomics interactions" if "search pgx" in command else "") for command in related)
    return "\n".join(lines) + "\n"

batch_modes = {
    "default": ([], [default, default_trast]),
    "all": (["--sections", "all"], [all_commands, all_trast]),
    "multi": (["--sections", "label,targets"], [multi, multi_trast]),
}
request_counts = {"default": 10, "all": 15, "multi": 8}
single_request_counts = {"default": 5, "all": 8, "multi": 4}
single_options = {"default": [], "all": ["all"], "multi": ["label", "targets"]}
identities = ["pembrolizumab", "trastuzumab"]

def reset_log():
    open(log, "w", encoding="utf-8").close()
    assert open(log, encoding="utf-8").read() == ""

def batch_item(markdown, identity):
    start = markdown.index("## " + identity + " — ok\n\n")
    end = markdown.find("\n\n---\n\n", start)
    if end < 0:
        end = markdown.index("\n## Summary", start)
    return markdown[start:end]

for mode, (options, expected_items) in batch_modes.items():
    reset_log()
    batch_json = json.loads(cli("--json", "batch", "drug", "pembrolizumab,trastuzumab", *options))
    assert batch_json["summary"] == {"total": 2, "succeeded": 2, "failed": 0}
    assert [item["input"] for item in batch_json["items"]] == identities
    assert [item["result"]["name"] for item in batch_json["items"]] == identities
    assert [item["result"]["_meta"]["next_commands"] for item in batch_json["items"]] == expected_items
    assert len(open(log, encoding="utf-8").readlines()) == request_counts[mode]
    reset_log()
    batch_markdown = cli("batch", "drug", "pembrolizumab,trastuzumab", *options)
    assert batch_markdown.index("## pembrolizumab — ok") < batch_markdown.index("## trastuzumab — ok")
    for identity, commands in zip(identities, expected_items):
        assert guidance(batch_item(batch_markdown, identity)) == expected_guidance(identity, mode, commands)
    assert len(open(log, encoding="utf-8").readlines()) == request_counts[mode]

for mode in batch_modes:
    for json_output in (False, True):
        reset_log()
        cli(*( ["--json"] if json_output else []), "get", "drug", "pembrolizumab", *single_options[mode])
        assert len(open(log, encoding="utf-8").readlines()) == single_request_counts[mode]

with tempfile.NamedTemporaryFile() as missing_ema:
    recovery_env = {"BIOMCP_EMA_DIR": missing_ema.name}
    recovered = json_card("get", "drug", "pembrolizumab", "safety", "--region", "eu", extra_env=recovery_env)
    assert recovered["section_outcomes"]["safety"]["outcome"] == "unavailable"
    assert recovered["_meta"]["next_commands"] == eu
    shell(recovered["_meta"]["next_commands"][0], recovery_env)
    recovery_markdown = cli("get", "drug", "pembrolizumab", "safety", "--region", "eu", extra_env=recovery_env)
    assert recovery_markdown.count("Retry: `biomcp get drug pembrolizumab safety --region eu`") == 1
    eu_guidance = """More:
  biomcp get drug pembrolizumab approvals   - Drugs@FDA approval history
  biomcp get drug pembrolizumab label   - approved-indication and FDA label detail beyond the base card
  biomcp get drug pembrolizumab regulatory --region eu   - approval and supplement history; use only if the base card lacks approval context

All:
  biomcp get drug pembrolizumab all --region eu
See also:
  biomcp search article --drug pembrolizumab --type review --limit 5   - supplement sparse structured data with review literature for indication context
  biomcp drug trials pembrolizumab
  biomcp drug adverse-events pembrolizumab   - inspect safety reports and adverse-event signal
  biomcp search pgx -d pembrolizumab   - pharmacogenomics interactions
  biomcp get gene PDCD1
"""
    assert guidance(recovery_markdown) == eu_guidance

# Parse and execute representative commands emitted by the production JSON owner.
for command in default:
    if command.endswith((" approvals", " all --region us", "adverse-events pembrolizumab")):
        shell(command)

with tempfile.NamedTemporaryFile(delete=False) as sentinel:
    sentinel_path = sentinel.name
os.unlink(sentinel_path)
hostile = "ticket1151 hostile  'x' \"q\" \\ $x `rm` ; & $(touch " + sentinel_path + ") path"
with open(log, "w", encoding="utf-8"):
    pass
hostile_card = json_card("get", "drug", "--name", hostile)
hostile_command = hostile_card["_meta"]["next_commands"][0]
shell(hostile_command)
assert not os.path.exists(sentinel_path)
mychem_lines = [line for line in open(log, encoding="utf-8") if line.startswith("GET /mychem/")]
assert len(mychem_lines) == 2
from urllib.parse import parse_qs, urlparse
for line in mychem_lines:
    assert parse_qs(urlparse(line.split(" ", 1)[1]).query)["q"][0] == hostile

def rpc(proc, request):
    proc.stdin.write(json.dumps(request) + "\n")
    proc.stdin.flush()
    ready, _, _ = select.select([proc.stdout], [], [], 60)
    if not ready:
        raise TimeoutError("MCP response deadline exceeded")
    return json.loads(proc.stdout.readline())

def start_server(server_env):
    cache = tempfile.TemporaryDirectory()
    child_env = server_env | {"BIOMCP_CACHE_DIR": cache.name}
    proc = subprocess.Popen(
        [binary, "serve"], cwd=os.environ["PWD"], env=child_env,
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        text=True,
    )
    try:
        rpc(proc, {"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{"protocolVersion":"2025-03-26", "capabilities":{}, "clientInfo":{"name":"ticket-1151", "version":"1"}}})
        proc.stdin.write(json.dumps({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}) + "\n")
        proc.stdin.flush()
        return proc, cache
    except BaseException:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)
        cache.cleanup()
        raise

def stop_server(proc, cache):
    try:
        proc.terminate()
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=5)
    finally:
        cache.cleanup()

def mcp(proc, tool, arguments, request_id):
    result = rpc(proc, {"jsonrpc":"2.0", "id":request_id, "method":"tools/call", "params":{"name":tool, "arguments":arguments}})["result"]
    assert result.get("isError") is False
    return result["content"][0]["text"]

def same_output(left, right):
    # stdio CLI adds a final blank line; MCP content framing carries one newline.
    return left.rstrip("\n") == right.rstrip("\n")

server, server_cache = start_server(env)
try:
    request_id = 2
    for json_output in (False, True):
        cli_text = cli(*( ["--json"] if json_output else []), "get", "drug", "pembrolizumab")
        raw_text = mcp(server, "biomcp", {"command":"biomcp get drug pembrolizumab", "json":json_output}, request_id)
        request_id += 1
        typed_text = mcp(server, "get", {"entity":"drug", "id":"pembrolizumab", "sections":[], "json":json_output}, request_id)
        request_id += 1
        assert same_output(raw_text, cli_text) and same_output(typed_text, cli_text)
    for mode, (options, expected_items) in batch_modes.items():
        batch_cli = cli("--json", "batch", "drug", "pembrolizumab,trastuzumab", *options)
        batch_payload = json.loads(batch_cli)
        assert [item["result"]["_meta"]["next_commands"] for item in batch_payload["items"]] == expected_items
        assert same_output(mcp(server, "biomcp", {"command":"biomcp batch drug pembrolizumab,trastuzumab" + (" " + " ".join(options) if options else ""), "json":True}, request_id), batch_cli)
        request_id += 1
        batch_cli_markdown = cli("batch", "drug", "pembrolizumab,trastuzumab", *options)
        assert same_output(mcp(server, "biomcp", {"command":"biomcp batch drug pembrolizumab,trastuzumab" + (" " + " ".join(options) if options else ""), "json":False}, request_id), batch_cli_markdown)
        request_id += 1
        for identity, commands in zip(identities, expected_items):
            assert guidance(batch_item(batch_cli_markdown, identity)) == expected_guidance(identity, mode, commands)
    tools = rpc(server, {"jsonrpc":"2.0", "id":request_id, "method":"tools/list", "params":{}})["result"]["tools"]
    get_schema = next(tool["inputSchema"] for tool in tools if tool["name"] == "get")
    drug_branch = next(branch for branch in get_schema["oneOf"] if branch["properties"]["entity"]["const"] == "drug")
    assert [tool["name"] for tool in tools] == ["biomcp", "search", "get", "variant_normalize_car", "variant_erepo", "gene_cspec", "variant_articles"]
    assert drug_branch["properties"]["sections"]["items"]["enum"] == ["label", "regulatory", "safety", "shortage", "targets", "indications", "interactions", "civic", "approvals", "all"]
    assert not any(tool["name"] == "batch" for tool in tools)
finally:
    stop_server(server, server_cache)

with tempfile.NamedTemporaryFile() as missing_ema:
    recovery_cli = cli("--json", "get", "drug", "pembrolizumab", "safety", "--region", "eu", extra_env={"BIOMCP_EMA_DIR": missing_ema.name})
    recovery_server, recovery_cache = start_server(env | {"BIOMCP_EMA_DIR": missing_ema.name})
    try:
        recovery_raw = mcp(recovery_server, "biomcp", {"command":"biomcp get drug pembrolizumab safety --region eu", "json":True}, 2)
        assert same_output(recovery_raw, recovery_cli)
        # Typed get has no region field; interactions is the fixture's unavailable
        # section, proving unavailable data is still a successful tool result.
        recovery_result = rpc(recovery_server, {"jsonrpc":"2.0", "id":1, "method":"tools/call", "params":{"name":"get", "arguments":{"entity":"drug", "id":"pembrolizumab", "sections":["interactions"], "json":True}}})["result"]
        assert recovery_result.get("isError") is False
        recovery_payload = json.loads(recovery_result["content"][0]["text"])
        assert recovery_payload["section_outcomes"]["interactions"]["outcome"] == "unavailable"
    finally:
        stop_server(recovery_server, recovery_cache)

print("drug-card projection production matrix passed")
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
