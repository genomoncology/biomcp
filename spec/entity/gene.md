# Gene Queries

Gene search is the fastest way to anchor a BioMCP session in a stable entity.
These canaries keep the focus on durable identity, deepen-path guidance, and
opt-in sections instead of volatile upstream counts or copy-edit trivia.

## Symbol-Based Search

Symbol search should still surface the canonical BRAF row in a human-scannable
table before the user pivots into deeper sections.

```bash
../../tools/biomcp-ci search gene BRAF --limit 3 | mustmatch like '# Genes: BRAF
B-Raf proto-oncogene'
```

## Exact Symbol Ranking and Pagination

An exact canonical symbol is promoted from the complete bounded provider set,
with duplicates removed before pagination and follow-up commands built from the
first retained row.

```bash
for offset in 0 1 2 3; do
  actual="$(../../tools/biomcp-ci --json search gene ODC1 --limit 1 --offset "$offset")"
  case "$offset" in
    0) expected='["ODC1"]' ;;
    1) expected='["SLC25A21"]' ;;
    2) expected='["OAZ1"]' ;;
    3) expected='[]' ;;
  esac
  jq -e --argjson expected "$expected" '.results | map(.symbol) == $expected' \
    <<<"$actual" | mustmatch 'true'
  jq -e --argjson offset "$offset" \
    '.count == (if $offset == 3 then 0 else 1 end) and .pagination == {"offset":$offset,"limit":1,"returned":(if $offset == 3 then 0 else 1 end),"total":3,"has_more":($offset < 2),"next_page_token":null} and (if $offset == 3 then ._meta == null else ._meta.next_commands == (if $offset == 0 then ["biomcp get gene ODC1","biomcp list gene"] elif $offset == 1 then ["biomcp get gene SLC25A21","biomcp list gene"] else ["biomcp get gene OAZ1","biomcp list gene"] end) end)' \
    <<<"$actual" | mustmatch 'true'
done
for query in odc1 ' OdC1 ' $'\u2003OdC1\u2003'; do
  ../../tools/biomcp-ci --json search gene "$query" --limit 2 \
    | jq -e '.results | map(.symbol) == ["ODC1", "SLC25A21"]' \
    | mustmatch 'true'
done
../../tools/biomcp-ci --json search gene ODC1 --limit 2 --offset 1 \
  | jq -e '(.results | map(.symbol)) == ["SLC25A21", "OAZ1"] and .count == 2 and .pagination.total == 3' \
  | mustmatch 'true'
../../tools/biomcp-ci --json search gene ODC1 --limit 1 \
  | jq -e '._meta.next_commands == ["biomcp get gene ODC1", "biomcp list gene"]' \
  | mustmatch 'true'
../../tools/biomcp-ci search gene ODC1 --limit 1 \
  | mustmatch like 'ODC1
Showing 1-1 of 3 results. Use --offset 1 for more.'

# Count each unique provider request so concurrent specs cannot perturb the
# exact acquisition deltas, including the second request made only for a
# positive-offset overflow page.
log="$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG"
request_count() {
  grep -Fc "$1" "$log" || true
}
request_count_window() {
  grep -F "$1" "$log" | grep -Fc "$2" || true
}
total50_query='GET /mygene/v3/query?q=%28symbol%3ATOTAL50+OR+alias%3ATOTAL50%29'
total51_query='GET /mygene/v3/query?q=%28symbol%3ATOTAL51+OR+alias%3ATOTAL51%29'
before_total50="$(request_count "$total50_query")"
before_total50_window="$(request_count_window "$total50_query" '&size=50&from=0')"
../../tools/biomcp-ci --json search gene TOTAL50 --limit 1 \
  | jq -e '(.results | map(.symbol)) == ["TOTAL50"] and .pagination.total == 50' \
  | mustmatch 'true'
test "$(( $(request_count "$total50_query") - before_total50 ))" -eq 1
test "$(( $(request_count_window "$total50_query" '&size=50&from=0') - before_total50_window ))" -eq 1
before_total51="$(request_count "$total51_query")"
../../tools/biomcp-ci --json search gene TOTAL51 --limit 1 \
  | jq -e '(.results | map(.symbol)) == ["OVERFLOW_DUP"] and .pagination.total == 51' \
  | mustmatch 'true'
test "$(( $(request_count "$total51_query") - before_total51 ))" -eq 1
before_total51="$(request_count "$total51_query")"
before_total51_first_window="$(request_count_window "$total51_query" '&size=50&from=0')"
before_total51_second_window="$(request_count_window "$total51_query" '&size=1&from=1')"
../../tools/biomcp-ci --json search gene TOTAL51 --limit 1 --offset 1 \
  | jq -e '(.results | map(.symbol)) == ["OVERFLOW_DUP"] and .pagination.total == 51' \
  | mustmatch 'true'
test "$(( $(request_count "$total51_query") - before_total51 ))" -eq 2
test "$(( $(request_count_window "$total51_query" '&size=50&from=0') - before_total51_first_window ))" -eq 1
test "$(( $(request_count_window "$total51_query" '&size=1&from=1') - before_total51_second_window ))" -eq 1
before_last_window="$(request_count_window "$total51_query" '&size=1&from=9999')"
../../tools/biomcp-ci --json search gene TOTAL51 --limit 1 --offset 9999 >/dev/null
test "$(( $(request_count_window "$total51_query" '&size=1&from=9999') - before_last_window ))" -eq 1
before_last_invalid="$(request_count "$total51_query")"
set +e
../../tools/biomcp-ci --json search gene TOTAL51 --limit 2 --offset 9999 >/dev/null
status=$?
set -e
test "$status" -eq 2
test "$(( $(request_count "$total51_query") - before_last_invalid ))" -eq 0
before_alk="$(request_count 'GET /mygene/v3/query?q=ALK+%5C%28fusion%5C%29')"
../../tools/biomcp-ci --json search gene 'ALK (fusion)' --limit 1 >/dev/null
test "$(( $(request_count 'GET /mygene/v3/query?q=ALK+%5C%28fusion%5C%29') - before_alk ))" -eq 1

# Local filters run against the complete retained set; provider predicates are
# present in the outbound query while their fixture response is deterministic.
../../tools/biomcp-ci --json search gene FILTERCASE --type protein-coding --chromosome chr7 --region chr7:150-250 \
  | jq -e '((.results | map(.symbol)) == ["FILTERCASE","FILTER_ALIAS"]) and .count == 2 and .pagination.total == 2' \
  | mustmatch 'true'
../../tools/biomcp-ci --json search gene FILTERCASE --type ncRNA \
  | jq -e '((.results | map(.symbol)) == ["FILTER_WRONG_TYPE"]) and .pagination.total == 1' \
  | mustmatch 'true'
../../tools/biomcp-ci --json search gene FILTERCASE --region chr7:300-400 \
  | jq -e '.results == [] and .count == 0 and .pagination.total == 0' \
  | mustmatch 'true'
../../tools/biomcp-ci --json search gene FILTERCASE --chromosome CHR7 \
  | jq -e '(.results | map(.symbol)) == ["FILTERCASE","FILTER_WRONG_TYPE","FILTER_OUTSIDE","FILTER_ALIAS"]' \
  | mustmatch 'true'
pathway_request='GET /mygene/v3/query?q=%28symbol%3AFILTERCASE+OR+alias%3AFILTERCASE%29+AND+%28pathway.kegg.id%3A%22R%5C-HSA%5C-5673001%22+OR+pathway.reactome.id%3A%22R%5C-HSA%5C-5673001%22+OR+pathway.kegg.name%3A*R%5C-HSA%5C-5673001*%29&species'
before_pathway="$(request_count "$pathway_request")"
../../tools/biomcp-ci --json search gene FILTERCASE --pathway R-HSA-5673001 \
  | jq -e '(.results | map(.symbol)) == ["FILTERCASE","FILTER_WRONG_TYPE","FILTER_WRONG_CHR","FILTER_OUTSIDE","FILTER_ALIAS"]' \
  | mustmatch 'true'
test "$(( $(request_count "$pathway_request") - before_pathway ))" -eq 1
go_request='GET /mygene/v3/query?q=%28symbol%3AFILTERCASE+OR+alias%3AFILTERCASE%29+AND+%28go.BP.id%3A%22GO%5C%3A0004672%22+OR+go.CC.id%3A%22GO%5C%3A0004672%22+OR+go.MF.id%3A%22GO%5C%3A0004672%22%29&species'
before_go="$(request_count "$go_request")"
../../tools/biomcp-ci --json search gene FILTERCASE --go GO:0004672 \
  | jq -e '(.results | map(.symbol)) == ["FILTERCASE","FILTER_WRONG_TYPE","FILTER_WRONG_CHR","FILTER_OUTSIDE","FILTER_ALIAS"]' \
  | mustmatch 'true'
test "$(( $(request_count "$go_request") - before_go ))" -eq 1

# Alias-only and no-exact queries retain canonical identity without inventing a
# provider row, while Lucene metacharacters remain escaped free-text terms.
../../tools/biomcp-ci --json search gene ALIASONLY --limit 1 \
  | jq -e '(.results | map(.symbol)) == ["CANONICAL_ALIAS"] and .pagination.total == 1' \
  | mustmatch 'true'
noexact_query='GET /mygene/v3/query?q=%28symbol%3ANOEXACT+OR+alias%3ANOEXACT%29'
before_noexact="$(request_count "$noexact_query")"
before_noexact_window="$(request_count_window "$noexact_query" '&size=50&from=0')"
../../tools/biomcp-ci --json search gene NOEXACT --limit 6 \
  | jq -e '((.results | map({symbol,entrez_id})) == [{symbol:"",entrez_id:""},{symbol:"NOEXACT_A",entrez_id:""},{symbol:"",entrez_id:"9303"},{symbol:"NOEXACT_B",entrez_id:"9304"},{symbol:"",entrez_id:""}]) and .count == 5 and .pagination.total == 5' \
  | mustmatch 'true'
test "$(( $(request_count "$noexact_query") - before_noexact ))" -eq 1
test "$(( $(request_count_window "$noexact_query" '&size=50&from=0') - before_noexact_window ))" -eq 1
before_no_exact="$(request_count 'GET /mygene/v3/query?q=%28symbol%3ANOTAREALGENE1091+OR+alias%3ANOTAREALGENE1091%29')"
../../tools/biomcp-ci --json search gene NOTAREALGENE1091 --limit 1 \
  | jq -e '.results == [] and .pagination.total == 0' \
  | mustmatch 'true'
test "$(( $(request_count 'GET /mygene/v3/query?q=%28symbol%3ANOTAREALGENE1091+OR+alias%3ANOTAREALGENE1091%29') - before_no_exact ))" -eq 1
before_braf_escape="$(request_count 'GET /mygene/v3/query?q=BRAF%5C%3AV600E')"
before_alk_escape="$(request_count 'GET /mygene/v3/query?q=ALK+%5C%28fusion%5C%29')"
for query in 'BRAF:V600E' 'ALK (fusion)'; do
  ../../tools/biomcp-ci --json search gene "$query" --limit 1 \
    | jq -e '.results == [] and .pagination.total == 0' \
    | mustmatch 'true'
done
test "$(( $(request_count 'GET /mygene/v3/query?q=BRAF%5C%3AV600E') - before_braf_escape ))" -eq 1
test "$(( $(request_count 'GET /mygene/v3/query?q=ALK+%5C%28fusion%5C%29') - before_alk_escape ))" -eq 1
lucene_query='+-=&&||><!(){}[]^"~*?:/\'
../../tools/biomcp-ci --json search gene "$lucene_query" --limit 1 \
  | jq -e '.results == [] and .pagination.total == 0' \
  | mustmatch 'true'

# Empty/invalid windows reject before any provider work.
before_invalid_window="$(request_count 'GET /mygene/v3/query?q=%28symbol%3AODC1+OR+alias%3AODC1%29')"
set +e
../../tools/biomcp-ci --json search gene ODC1 --limit 1 --offset 10000 >/dev/null
status=$?
set -e
test "$status" -eq 2
test "$(( $(request_count 'GET /mygene/v3/query?q=%28symbol%3AODC1+OR+alias%3AODC1%29') - before_invalid_window ))" -eq 0
before_invalid_limit="$(request_count 'GET /mygene/v3/query?q=%28symbol%3AODC1+OR+alias%3AODC1%29')"
set +e
../../tools/biomcp-ci --json search gene ODC1 --limit 51 >/dev/null
status=$?
set -e
test "$status" -eq 2
test "$(( $(request_count 'GET /mygene/v3/query?q=%28symbol%3AODC1+OR+alias%3AODC1%29') - before_invalid_limit ))" -eq 0
```

## Raw and Typed MCP Gene Search Match the CLI

Both MCP search surfaces reuse the CLI's ordered rows and follow-up commands;
MCP adds only its existing next-command section to Markdown.

```bash
python3 - <<'PY' | mustmatch like 'raw and typed gene search converges'
import json, os, subprocess

binary = os.environ["BIOMCP_BIN"]
env = os.environ.copy()

def cli(args, json_output):
    command = [binary] + (["--json"] if json_output else []) + args
    return subprocess.check_output(command, text=True, env=env)

proc = subprocess.Popen([binary, "serve"], stdin=subprocess.PIPE,
                        stdout=subprocess.PIPE, text=True, env=env)
def call(identifier, name, arguments):
    proc.stdin.write(json.dumps({"jsonrpc": "2.0", "id": identifier,
        "method": "tools/call", "params": {"name": name,
        "arguments": arguments}}) + "\n")
    proc.stdin.flush()
    result = json.loads(proc.stdout.readline())["result"]
    assert result.get("isError") is not True
    return result["content"][0]["text"]

try:
    proc.stdin.write(json.dumps({"jsonrpc": "2.0", "id": 1,
        "method": "initialize", "params": {"protocolVersion": "2025-03-26",
        "capabilities": {}, "clientInfo": {"name": "spec", "version": "1"}}}) + "\n")
    proc.stdin.flush()
    json.loads(proc.stdout.readline())
    proc.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}\n')
    proc.stdin.flush()
    for identifier, offset in enumerate((0, 1), 2):
        args = ["search", "gene", "ODC1", "--limit", "1", "--offset", str(offset)]
        cli_markdown = cli(args, False)
        if offset == 0:
            expected_markdown = """# Genes: ODC1

Found 1 gene

| Symbol | Name | Entrez ID | Coordinate | Build | UniProt | OMIM |
|---|---|---|---|---|---|---|
| ODC1 | ornithine decarboxylase 1 | 4953 | - | - | none | none |


Use `get gene <symbol>` for details.
Filters: -q <query>, --type <protein-coding|ncRNA|pseudo>, --chromosome <N>, --region <chr:start-end>, --pathway <id>, --go <term>


Showing 1-1 of 3 results. Use --offset 1 for more.


"""
        else:
            expected_markdown = """# Genes: ODC1, offset=1

Found 1 gene

| Symbol | Name | Entrez ID | Coordinate | Build | UniProt | OMIM |
|---|---|---|---|---|---|---|
| SLC25A21 | solute carrier family 25 member 21 | 23530 | - | - | none | none |


Use `get gene <symbol>` for details.
Filters: -q <query>, --type <protein-coding|ncRNA|pseudo>, --chromosome <N>, --region <chr:start-end>, --pathway <id>, --go <term>


Showing 2-2 of 3 results. Use --offset 2 for more.


"""
        assert cli_markdown == expected_markdown
        raw_markdown = call(identifier, "biomcp", {"command": "biomcp " + " ".join(args)})
        typed_markdown = call(identifier + 10, "search", {"entity": "gene", "query": "ODC1",
            "limit": 1, "offset": offset})
        table = lambda text: [line for line in text.splitlines() if line.startswith("|")]
        assert table(raw_markdown) == table(typed_markdown) == table(cli_markdown)
        assert raw_markdown == typed_markdown
        expected_command = "biomcp get gene " + ("ODC1" if offset == 0 else "SLC25A21")
        expected_footer = ("\n## Next commands\n- `" + expected_command
                          + "`\n- `biomcp list gene`")
        assert raw_markdown == expected_markdown + expected_footer
        cli_json_text = cli(args, True)
        cli_json = json.loads(cli_json_text)
        raw_json = call(identifier + 20, "biomcp", {"command": "biomcp " + " ".join(args), "json": True})
        typed_json = call(identifier + 30, "search", {"entity": "gene", "query": "ODC1",
            "limit": 1, "offset": offset, "json": True})
        raw_value = json.loads(raw_json)
        typed_value = json.loads(typed_json)
        assert raw_value["results"] == typed_value["results"] == cli_json["results"]
        assert raw_value["count"] == typed_value["count"] == cli_json["count"]
        assert raw_value["pagination"] == typed_value["pagination"] == cli_json["pagination"]
        assert raw_value["_meta"] == typed_value["_meta"] == cli_json["_meta"]
        assert raw_value["results"] == ([{"symbol": "ODC1", "name": "ornithine decarboxylase 1", "entrez_id": "4953", "genomic_coordinates": None, "uniprot_id": None, "omim_id": None}] if offset == 0 else [{"symbol": "SLC25A21", "name": "solute carrier family 25 member 21", "entrez_id": "23530", "genomic_coordinates": None, "uniprot_id": None, "omim_id": None}])
        assert raw_value["_meta"]["next_commands"] == (["biomcp get gene ODC1", "biomcp list gene"] if offset == 0 else ["biomcp get gene SLC25A21", "biomcp list gene"])
finally:
    proc.terminate()
    proc.wait(timeout=5)
print("raw and typed gene search converges")
PY
```

## Adversarial identity and public-surface proof

The production parser receives the exact emitted follow-up command, including a
provider symbol that exercises shell quoting. It must remain one argument and
gene-get validation must reject it before a second provider request.

```bash
python3 - <<'PY' | mustmatch like 'hostile gene command is inert and public surfaces are unchanged'
import json, os, shlex, subprocess

binary = os.environ["BIOMCP_BIN"]
env = os.environ.copy()
log = os.environ["BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG"]
def hostile_requests():
    return [line for line in open(log, encoding="utf-8")
            if "q=%28symbol%3AHOSTILE+OR+alias%3AHOSTILE%29" in line]
before_search = hostile_requests()
payload = json.loads(subprocess.check_output(
    [binary, "--json", "search", "gene", "HOSTILE", "--limit", "1"], text=True))
hostile_symbol = 'bad " quote \\ slash $HOME `tick`; &amp'
assert payload["results"][0]["symbol"] == hostile_symbol
assert len(hostile_requests()) - len(before_search) == 1
command = payload["_meta"]["next_commands"][0]
parts = shlex.split(command)
assert parts[:3] == ["biomcp", "get", "gene"] and len(parts) == 4
assert command == 'biomcp get gene "bad \\" quote \\\\ slash \\$HOME \\`tick\\`; &amp"'
before = hostile_requests()
result = subprocess.run(command.replace("biomcp", binary, 1), shell=True, text=True, capture_output=True, env=env)
assert result.returncode == 2
assert hostile_requests() == before

def mcp_tools():
    server = subprocess.Popen([binary, "serve"], stdin=subprocess.PIPE,
                              stdout=subprocess.PIPE, text=True, env=env)
    try:
        for message in (
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params":
             {"protocolVersion": "2025-03-26", "capabilities": {},
              "clientInfo": {"name": "spec", "version": "1"}}},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
        ):
            server.stdin.write(json.dumps(message) + "\n")
            server.stdin.flush()
            response = json.loads(server.stdout.readline())
            if message["id"] == 2:
                return response["result"]["tools"]
    finally:
        server.terminate()
        server.wait(timeout=5)

tools_before = mcp_tools()
tools_after = mcp_tools()
assert tools_before == tools_after
assert [tool["name"] for tool in tools_before] == [
    "biomcp", "search", "get", "variant_normalize_car", "variant_erepo",
    "gene_cspec", "variant_articles"]
assert len(tools_before) == 7
search_schema = next(tool["inputSchema"] for tool in tools_before if tool["name"] == "search")
gene_branch = next(branch for branch in search_schema["oneOf"]
                   if branch["properties"]["entity"] == {"const": "gene"})
assert gene_branch == {
    "additionalProperties": False,
    "anyOf": [{"required": ["query"]}, {"required": ["gene_type"]},
              {"required": ["chromosome"]}, {"required": ["region"]}],
    "properties": {
        "chromosome": {"maxLength": 256, "minLength": 1, "type": "string"},
        "entity": {"const": "gene"},
        "gene_type": {"maxLength": 256, "minLength": 1, "type": "string"},
        "json": {"default": False, "type": "boolean"},
        "limit": {"default": 10, "maximum": 25, "minimum": 1, "type": "integer"},
        "offset": {"default": 0, "maximum": 1000, "minimum": 0, "type": "integer"},
        "query": {"maxLength": 256, "minLength": 1, "type": "string"},
        "region": {"maxLength": 256, "minLength": 1, "type": "string"}},
    "required": ["entity"], "type": "object"}
help_before = subprocess.check_output([binary, "search", "gene", "--help"], text=True, env=env)
help_after = subprocess.check_output([binary, "search", "gene", "--help"], text=True, env=env)
assert help_before == help_after == """Search genes by symbol, name, type, or chromosome (MyGene.info)

Usage: biomcp search gene [OPTIONS] [QUERY]

Arguments:
  [QUERY]  Optional positional query alias for -q/--query

Options:
  -q, --query <QUERY>            Free text query (gene name, symbol, or keyword)
      --type <GENE_TYPE>         Filter by gene type (e.g., protein-coding, ncRNA, pseudo)
      --chromosome <CHROMOSOME>  Filter by chromosome (e.g., 7, X)
      --region <REGION>          Filter by genomic region (chr:start-end)
      --pathway <PATHWAY>        Filter by pathway ID/name (e.g., R-HSA-5673001)
      --go <GO_TERM>             Filter by GO term ID/text (e.g., GO:0004672)
  -l, --limit <LIMIT>            Maximum results, 1-50 (default: 10) [default: 10]
      --offset <OFFSET>          Skip the first N results [default: 0]
  -j, --json                     Output as JSON instead of Markdown
      --no-cache                 Use no managed request state: bypass HTTP cache and article sessions
  -h, --help                     Print help

EXAMPLES:
  biomcp search gene BRAF
  biomcp search gene -q kinase --type protein-coding --region chr7:140424943-140624564 --limit 5

See also: biomcp list gene
"""

print("hostile gene command is inert and public surfaces are unchanged")
PY
```

## Search Table Contract

The search surface needs to stay readable for humans and still expose machine
follow-ups through `_meta.next_commands`.

```bash
../../tools/biomcp-ci --json search gene BRAF --limit 3 | mustmatch like '"next_commands":'
../../tools/biomcp-ci --json search gene BRAF --limit 3 | jq -e '._meta.next_commands[0] | test("^biomcp get gene .+$")' >/dev/null
../../tools/biomcp-ci --json search gene BRAF --limit 3 | jq -e '._meta.next_commands | any(. == "biomcp list gene")' >/dev/null
```

## Identity Card

The default card should keep the persistent identifier and the progressive
disclosure hints that let readers deepen into the right follow-up section.

```bash
../../tools/biomcp-ci get gene BRAF | mustmatch like 'Entrez ID: 673
biomcp get gene BRAF pathways
biomcp get gene BRAF diagnostics'
```

## Shipped schema matches the fixture-backed CLI payload

The checked-in gene schema must validate the actual JSON emitted from the captured MyGene BRAF response. This closes the gate if the Rust serializer and shipped skill schema drift apart, even when the static example still agrees with the stale schema.

```bash
gene_json="$(../../tools/biomcp-ci --json get gene BRAF)"
gencc_gene_json="$(../../tools/biomcp-ci --json get gene ODC1 gencc)"
GENE_JSON="$gene_json" GENCC_GENE_JSON="$gencc_gene_json" uv run --no-sync python3 - ../../skills/schemas/gene.json <<'PY' | mustmatch like 'gene schema matches fixture-backed CLI payload'
import json
import os
from copy import deepcopy
from pathlib import Path
import sys

from jsonschema import Draft202012Validator, ValidationError

schema = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
Draft202012Validator.check_schema(schema)
validator = Draft202012Validator(schema)
payload = json.loads(os.environ["GENE_JSON"])
validator.validate(payload)
validator.validate(json.loads(os.environ["GENCC_GENE_JSON"]))

without_coordinates = deepcopy(payload)
without_coordinates.pop("genomic_coordinates")
validator.validate(without_coordinates)

null_coordinates = deepcopy(payload)
null_coordinates["genomic_coordinates"] = None
validator.validate(null_coordinates)

def require_rejection(candidate, label):
    try:
        validator.validate(candidate)
    except ValidationError:
        return
    raise AssertionError(f"gene schema accepted invalid {label}")

for required_field in ("coordinate", "genome_build", "source"):
    missing_required = deepcopy(payload)
    missing_required["genomic_coordinates"].pop(required_field)
    require_rejection(missing_required, f"coordinate object without {required_field}")

null_provenance = deepcopy(payload)
null_provenance["genomic_coordinates"]["provenance"] = None
require_rejection(null_provenance, "null coordinate provenance")

extra_property = deepcopy(payload)
extra_property["genomic_coordinates"]["unexpected"] = "value"
require_rejection(extra_property, "extra coordinate property")
print("gene schema matches fixture-backed CLI payload")
PY
```

## Common Alias Get Resolves Canonical Gene

Clinical reports and papers often use common aliases instead of the HGNC symbol.
For an alias that maps to one canonical gene, `get gene` should return the same
stable gene card a user would get from the official symbol.

```bash
../../tools/biomcp-ci --json get gene PD-L1 | mustmatch like '"symbol": "CD274"
"entrez_id": "29126"
"PD-L1"'
```

## Diagnostics and Pathways Pivots

The base gene view advertises its diagnostic and pathway deepen paths without
requiring any optional enrichment provider.

```bash
../../tools/biomcp-ci --json get gene BRCA1 | mustmatch like '"next_commands":'
../../tools/biomcp-ci --json get gene BRCA1 | jq -e '._meta.next_commands | any(. == "biomcp get gene BRCA1 diagnostics")' >/dev/null
../../tools/biomcp-ci --json get gene BRCA1 | jq -e '._meta.next_commands | any(. == "biomcp get gene BRCA1 pathways")' >/dev/null
```

## Observed MyGene Requests

The local fixture records requests emitted by the production client, including
the bounded search and exact-symbol identity plans.

```bash
grep -F 'GET /mygene/v3/query?q=%28symbol%3ABRAF+OR+alias%3ABRAF%29' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" \
  | grep -Fq '&size=50&from=0'
grep -F 'GET /mygene/v3/query?q=symbol%3A%22BRCA1%22' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'symbol%3A%22BRCA1%22'
```

## Typed optional-section outcomes

Requested sections keep a bounded state even when providers return no rows or
are temporarily unavailable. Provenance carries the same state rather than
inferring success from an empty collection.

```bash
../../tools/biomcp-ci --json get gene BRAF go interactions \
  | jq '. as $root | ["go", "interactions"] | all(.[]; . as $key | $root.section_outcomes[$key] as $outcome | ($outcome.outcome | IN("data", "empty", "unavailable")) and ($root._meta.section_sources | any(.key == $key and .outcome == $outcome.outcome and .sources == $outcome.sources)) and ($root._meta.section_sources | all(.key != $key or (.outcome == $outcome.outcome and .sources == $outcome.sources))))' \
  | mustmatch 'true'
grep -F 'GET /quickgo/QuickGO/services/annotation/search?geneProductId=P15056&limit=20' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'geneProductId=P15056'
grep -F 'GET /string/api/json/network?identifiers=BRAF&species=9606&limit=15' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'species=9606'
```

## Partial ClinGen evidence

ClinGen validity, dosage sensitivity, and the shared gene lookup start together
under independent deadlines. A slow dosage download keeps completed validity
evidence and reports the exact degraded aggregate on both provenance surfaces.

```bash
BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE" BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/dosage-timeout" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=40 \
  ../../tools/biomcp-ci --json get gene TP53 clingen \
  | jq -e '.clingen.validity[0].disease == "Li-Fraumeni syndrome" and .clingen.validity_status == {status:"data", op:"gene_validity_download"} and .clingen.dosage_status == {status:"timed_out", op:"gene_dosage_download", message:"ClinGen gene-dosage download timed out."} and .section_outcomes.clingen == {outcome:"degraded", sources:["ClinGen"], message:"ClinGen gene evidence is partial; one result family is unavailable."} and (._meta.section_sources | any(.key == "clingen" and .label == "ClinGen" and .outcome == "degraded" and .sources == ["ClinGen"]))' \
  | mustmatch 'true'
```

The inverse failure retains the newest dosage row, including a literal ClinGen
no-evidence classification. Markdown exposes both family states and never
invents a missing classification.

```bash
BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE" BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/validity-fail" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 \
  ../../tools/biomcp-ci --json get gene TP53 clingen \
  | jq -e '.clingen.validity_status == {status:"failed", op:"gene_validity_download", message:"ClinGen gene-validity download failed."} and .clingen.dosage_status == {status:"data", op:"gene_dosage_download"} and .clingen.haploinsufficiency == "Sufficient Evidence for Haploinsufficiency" and .clingen.triplosensitivity == "No Evidence for Triplosensitivity" and .section_outcomes.clingen.outcome == "degraded"' \
  | mustmatch 'true'
BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE" BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/validity-fail" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 \
  ../../tools/biomcp-ci get gene TP53 clingen \
  | mustmatch like 'Gene-Disease Validity Status
Status: `failed`
ClinGen gene-validity download failed.
Dosage-Sensitivity Status
Status: `data`
Haploinsufficiency: Sufficient Evidence for Haploinsufficiency
Triplosensitivity: No Evidence for Triplosensitivity'
```

Raw MCP text and JSON plus typed MCP `get` travel through the same section
contract. The typed request schema remains the existing gene/section request.

```bash
BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE" BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen/validity-fail" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 \
  BIOMCP_CACHE_DIR="${BIOMCP_PROVIDER_CONTRACT_READY_FILE%/base-url}/clingen-mcp-cache" \
  bash ../fixtures/run-section-outcome-mcp.sh ../.. clingen-surfaces \
  | mustmatch like 'RAW TEXT
Gene-Disease Validity Status
ClinGen gene-validity download failed.
Triplosensitivity: No Evidence for Triplosensitivity
RAW JSON
"validity_status": {
"status": "failed"
"dosage_status": {
"status": "data"
TYPED JSON
"section_outcomes": {
"clingen": {
"outcome": "degraded"
"section_sources":'
```

## GenCC submission-level validity

The receipt-backed new-format fixture preserves the three independent ODC1
assertions. JSON, Markdown, all-section, and batch surfaces keep their
evaluated-date order and source-specific lifecycle state without claiming a
consensus.

```bash
export BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE"
../../tools/biomcp-ci list gene | mustmatch like 'get gene <symbol> gencc`
GenCC submission-level gene-disease validity'
../../tools/biomcp-ci get gene --help | mustmatch like 'clingen, gencc, constraint'
../../tools/biomcp-ci gencc --help | mustmatch like 'sync  Revalidate the local GenCC gene-disease validity dataset'
../../tools/biomcp-ci --json get gene ODC1 gencc \
  | jq -e '.gencc.assertions | ((length == 3) and (map(.submitter.label) == ["G2P", "PanelApp Australia", "Labcorp Genetics (formerly Invitae)"]))' \
  | mustmatch 'true'
../../tools/biomcp-ci --json get gene ODC1 gencc \
  | jq -e '.gencc.total_matching_assertions == 3 and (.gencc.truncated | not) and .gencc.status.freshness == "fresh" and .gencc.status.result == "data" and .section_outcomes.gencc == {outcome:"data", sources:["GenCC"]} and (._meta.section_sources | any(.key == "gencc" and .label == "GenCC gene-disease validity" and .outcome == "data" and .sources == ["GenCC"]))' \
  | mustmatch 'true'
../../tools/biomcp-ci get gene ODC1 gencc \
  | mustmatch like '## GenCC gene-disease validity
G2P (GENCC:000112)
PanelApp Australia (GENCC:000111)
Labcorp Genetics (formerly Invitae) (GENCC:000106)
[PMID 30239107](<https://pubmed.ncbi.nlm.nih.gov/30239107/>)'
BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 ../../tools/biomcp-ci --json get gene ODC1 all \
  | jq -e '.gencc.assertions | length == 3' \
  | mustmatch 'true'
../../tools/biomcp-ci --json batch gene ODC1,ODC1 --sections gencc \
  | jq -e '.summary == {total:2,succeeded:2,failed:0} and (.items | all(.status == "ok" and (.result.gencc.assertions | length == 3)))' \
  | mustmatch 'true'
unset BIOMCP_TEST_UNPACED_ORIGIN
```

GenCC and ClinGen remain independent sections and provenance owners. The same
GenCC projection is available through raw MCP text/JSON and typed MCP `get`.

```bash
export BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE"
BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE" BIOMCP_CLINGEN_BASE="$BIOMCP_PROVIDER_CONTRACT_BASE/clingen" BIOMCP_GENE_OPTIONAL_TIMEOUT_MS=200 ../../tools/biomcp-ci --json get gene TP53 clingen gencc \
  | jq -e '.clingen.validity[0].disease == "Li-Fraumeni syndrome" and .gencc.status.result == "empty" and .section_outcomes.clingen.sources == ["ClinGen"] and .section_outcomes.gencc.sources == ["GenCC"]' \
  | mustmatch 'true'
BIOMCP_CACHE_DIR="${BIOMCP_PROVIDER_CONTRACT_READY_FILE%/base-url}/gencc-mcp-cache" \
  bash ../fixtures/run-section-outcome-mcp.sh ../.. gencc-surfaces \
  | jq -e '.raw_text | contains("## GenCC gene-disease validity") and contains("G2P (GENCC:000112)")' \
  | mustmatch 'true'
BIOMCP_CACHE_DIR="${BIOMCP_PROVIDER_CONTRACT_READY_FILE%/base-url}/gencc-mcp-cache" \
  bash ../fixtures/run-section-outcome-mcp.sh ../.. gencc-surfaces \
  | jq -e '.raw_json.gencc == .typed_json.gencc and .raw_json.section_outcomes.gencc == {outcome:"data", sources:["GenCC"]} and .typed_json.section_outcomes.gencc == .raw_json.section_outcomes.gencc and (.raw_json.gencc.assertions | map(.submitter.label)) == ["G2P", "PanelApp Australia", "Labcorp Genetics (formerly Invitae)"]' \
  | mustmatch 'true'
unset BIOMCP_TEST_UNPACED_ORIGIN
```

## GenCC adapter projection parity

The adapter's available, identity-match, and unavailable representatives are
enough to prove the public surfaces consume one projection. The four calls in
each representative compare the section payload, outcome, and provenance;
they do not repeat the full lifecycle matrix through every transport.

```bash
python3 - <<'PY' | mustmatch like 'GenCC adapter projections converge across CLI, raw MCP, typed MCP, and batch'
import json
import os
import shutil
import subprocess
import tempfile

binary = os.environ["BIOMCP_BIN"]
base_env = os.environ.copy()
base_env["BIOMCP_TEST_UNPACED_ORIGIN"] = base_env["BIOMCP_PROVIDER_CONTRACT_BASE"]

def cli(args, env):
    return json.loads(subprocess.check_output(
        [binary, "--json", *args], text=True, env=env))

def raw_and_typed(symbol, env):
    mcp_env = env.copy()
    mcp_env["BIOMCP_GENCC_SURFACES_GENE"] = symbol
    output = subprocess.check_output(
        ["bash", "../fixtures/run-section-outcome-mcp.sh", "../..", "gencc-surfaces"],
        text=True, env=mcp_env)
    surfaces = json.loads(output)
    return surfaces["raw_json"], surfaces["typed_json"]

def section_projection(card):
    return {
        "gencc": card["gencc"],
        "outcome": card["section_outcomes"]["gencc"],
        "provenance": [entry for entry in card["_meta"]["section_sources"]
                       if entry["key"] == "gencc"],
    }

def assert_surfaces(symbol, env, expected):
    cli_card = cli(["get", "gene", symbol, "gencc"], env)
    raw_card, typed_card = raw_and_typed(symbol, env)
    batch = cli(["batch", "gene", symbol, "--sections", "gencc"], env)
    batch_card = batch["items"][0]["result"]
    projections = [section_projection(card) for card in
                   (cli_card, raw_card, typed_card, batch_card)]
    assert all(projection == projections[0] for projection in projections), projections
    assert projections[0]["outcome"] == expected, projections[0]
    return projections[0]

# Available data: warm once so all four representative calls use local_query.
available_env = base_env.copy()
cli(["get", "gene", "ODC1", "gencc"], available_env)
available = assert_surfaces("ODC1", available_env,
    {"outcome": "data", "sources": ["GenCC"]})
assert len(available["gencc"]["assertions"]) == 3

# A resolved HGNC that belongs to a different symbol is identity_match, not an
# empty GenCC result. The fixture route supplies that conflicting MyGene card.
identity = assert_surfaces("GENCCIDENTITY", base_env,
    {"outcome": "unavailable", "sources": [],
     "message": "GenCC gene identity is inconclusive; no GenCC absence can be concluded."})
assert identity["gencc"]["status"]["operation"] == "identity_match"
assert identity["gencc"]["assertions"] == []

# Establish the failed acquisition once; subsequent calls share its durable
# retry-suppressed projection without multiplying the unavailable matrix.
unavailable_env = base_env.copy()
unavailable_env["BIOMCP_GENCC_BASE"] = base_env["BIOMCP_PROVIDER_CONTRACT_BASE"] + "/gencc/missing"
unavailable_env["BIOMCP_GENCC_TEST_NOW"] = "2026-09-09T00:00:00Z"
unavailable_dir = tempfile.mkdtemp(prefix="gencc-adapter-unavailable-",
                                    dir=base_env["BIOMCP_GENCC_FIXTURE_PARENT"])
unavailable_env["BIOMCP_GENCC_DIR"] = unavailable_dir
try:
    first = cli(["get", "gene", "ODC1", "gencc"], unavailable_env)
    assert first["gencc"]["status"]["freshness"] == "unavailable"
    unavailable = assert_surfaces("ODC1", unavailable_env,
        {"outcome": "unavailable", "sources": [],
         "message": "GenCC data is unavailable; no GenCC absence can be concluded."})
    assert unavailable["gencc"]["status"]["result"] == "unknown"
    assert unavailable["gencc"]["assertions"] == []
finally:
    shutil.rmtree(unavailable_dir)

print("GenCC adapter projections converge across CLI, raw MCP, typed MCP, and batch")
PY
```

Health uses a metadata-only HEAD. An explicit sync then revalidates the
immutable generation with both validators and a zero-body `304`; ordinary
fresh reuse above performs no second download.

```bash
export BIOMCP_TEST_UNPACED_ORIGIN="$BIOMCP_PROVIDER_CONTRACT_BASE"
../../tools/biomcp-ci --json health --api GenCC \
  | jq -e '.total == 1 and .healthy == 1 and .rows[0].api == "GenCC" and .rows[0].status == "ok"' \
  | mustmatch 'true'
grep -F 'HEAD /gencc/download/action/submissions-export-csv?format=new' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" \
  | mustmatch like 'format=new'
../../tools/biomcp-ci --json gencc sync | jq -e '.source == "gencc" and .status == "synchronized" and (.changed | not) and (has("updated") | not)' | mustmatch 'true'
grep -F 'GET /gencc/download/action/submissions-export-csv?format=new If-None-Match="6ebdbf28b305e99e349ed827a219214b" If-Modified-Since=Sun, 06 Sep 2026 06:00:29 GMT' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" \
  | mustmatch like 'If-None-Match="6ebdbf28b305e99e349ed827a219214b"'
test "$(grep -c '^GET /gencc/download/action/submissions-export-csv?format=new' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG")" -eq 2
unset BIOMCP_TEST_UNPACED_ORIGIN
```

## All-Section Warm Budget

Quarantined from routine executable specs by ticket 372 because this timing-only
canary failed twice during routine `make spec-pr` at 45599ms and 43332ms against
a 12000ms ceiling. Ticket 371's request-contract strategy keeps performance
canaries out of the default gate; restore this only as a deterministic
benchmark/ratchet or explicit performance lane.

## Tissue-Expression Context

Human Protein Atlas data belongs in an opt-in deepen path and retains its
source reliability and subcellular context.

```bash
../../tools/biomcp-ci get gene BRAF hpa | mustmatch like '## Human Protein Atlas
| Adipose tissue | Low |
Reliability: Supported
Subcellular main locations: cytosol, vesicles'
grep -F 'GET /hpa/ENSG00000157764.xml' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like 'ENSG00000157764.xml'
```

## Druggability & Targets

Targetability context stays separate from the default card while combining
Open Targets tractability and DGIdb interaction evidence.

```bash
../../tools/biomcp-ci get gene EGFR druggability | mustmatch like '## Druggability
OpenTargets tractability
| antibody | yes | Approved Drug'
grep -F 'POST /dgidb/api/graphql' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"gene":"EGFR"'
grep -F 'POST /opentargets/api/v4/graphql' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"ensemblId":"ENSG00000146648"'
```

## Funding

Funding remains opt-in and retains its source-attributed bounded table.

```bash
../../tools/biomcp-ci get gene ERBB2 funding | mustmatch like '## Funding (NIH Reporter)
| Project | PI | Organization | FY | Amount |
Showing top 8 unique grants from 187 matching NIH project-year records'
grep -F 'POST /nih/v2/projects/search' "$BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG" | mustmatch like '"search_text":"\"ERBB2\""'
```

## Diagnostic Local Data

The diagnostic deepen path consumes the bounded local GTR bundle rather than
downloading provider data during the routine gate.

```bash
../../tools/biomcp-ci get gene BRCA1 diagnostics | mustmatch like '## Diagnostics
GTR000000001.1
NCBI Genetic Testing Registry'
```
