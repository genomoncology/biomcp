# Phenotype Queries

Phenotype search turns symptom language or HPO IDs into a ranked disease shortlist. These captured contracts use the shipped CLI against fresh HPO and Monarch responses served by the supervised routine fixture.

## Captured Symptom-Phrase Route

The routine fixture resolves the symptom phrase through HPO search and then replays the exact Monarch similarity request produced by those identifiers.

```bash
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 | mustmatch like '# Phenotype Search: seizure, developmental delay
| Disease ID | Disease Name | Similarity Score |
MONDO:0007367
febrile seizures, familial, 1'
```

## Captured HPO-ID Route

Direct HPO IDs skip phrase resolution and use the same similarity and rendering path.

```bash
../../tools/biomcp-ci search phenotype 'HP:0001250 HP:0001263' --limit 3 | mustmatch like '# Phenotype Search: HP:0001250 HP:0001263
| Disease ID | Disease Name | Similarity Score |
MONDO:0010450
intellectual disability, X-linked 89'
```

## Disease Follow-Up Guidance

The captured phrase result teaches the typed disease command for its top match.

```bash
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 | mustmatch like 'See also:
biomcp get disease "febrile seizures, familial, 1" genes phenotypes'
```

## Captured JSON Follow-Up Envelope

JSON callers receive the same typed disease follow-up without an unsupported phenotype-get command.

```bash
../../tools/biomcp-ci --json search phenotype 'HP:0001250 HP:0001263' --limit 1 | jq '(.pagination.total == null) and .pagination.has_more and (.pagination.provider_window_limit == 50) and (.pagination.provider_raw_row_count == 50) and .pagination.provider_window_exhausted and (._meta.next_commands | any(startswith("biomcp search phenotype ") and endswith("--limit 1 --offset 1"))) and (._meta.next_commands | any(startswith("biomcp get disease ") and endswith(" genes phenotypes")))' | mustmatch 'true'
```

The symptom-phrase fixture is a short raw response, so the same metadata does
not claim that Monarch's 50-row window was exhausted.

```bash
../../tools/biomcp-ci --json search phenotype 'seizure, developmental delay' --limit 3 | jq '(.pagination.provider_window_limit == 50) and (.pagination.provider_raw_row_count == 3) and (.pagination.provider_window_exhausted == false)' | mustmatch 'true'
```

## Stable Provider Order And Local Pages

Every supported page is sliced from the same provider-ordered, normalized
window. The fixture includes tied scores, a later duplicate with a larger
score, and a non-disease row so sorting or counting after normalization cannot
accidentally satisfy the contract.

```bash
python3 - <<'PY' | mustmatch like 'phenotype pages share one fixed provider order'
import json, os, subprocess

binary = os.environ["BIOMCP_BIN"]
env = os.environ.copy()
log = os.environ["BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG"]
initial_lines = open(log, encoding="utf-8").readlines()

def search(limit, offset=0):
    result = subprocess.run(
        [binary, "--json", "search", "phenotype", "HP:0001250 HP:0001263", "--limit", str(limit), "--offset", str(offset)],
        check=True, capture_output=True, text=True, env=env,
    )
    return json.loads(result.stdout)

pages = {}
for limit in (1, 2, 3, 5):
    for offset in (0, 1):
        before = open(log, encoding="utf-8").read().count("/monarch/v3/api/semsim/")
        pages[(limit, offset)] = search(limit, offset)
        after = open(log, encoding="utf-8").read().count("/monarch/v3/api/semsim/")
        assert after == before + 1

ids = lambda page: [row["disease_id"] for row in page["results"]]
assert ids(pages[(1, 0)]) == ids(pages[(5, 0)])[:1]
assert ids(pages[(2, 0)]) == ids(pages[(5, 0)])[:2]
assert ids(pages[(3, 0)]) == ids(pages[(5, 0)])[:3]
page_two = search(3, 2)
assert ids(pages[(2, 0)]) + ids(page_two) == ids(pages[(5, 0)])
assert ids(pages[(5, 0)])[:4] == ["MONDO:0010450", "MONDO:0007367", "MONDO:0000002", "MONDO:0000001"]
assert pages[(5, 0)]["results"][0]["disease_name"] == "intellectual disability, X-linked 89"

requests = [line for line in open(log, encoding="utf-8").readlines()[len(initial_lines):] if "/monarch/v3/api/semsim/" in line]
assert requests and all(line.rstrip().endswith("?limit=50") for line in requests)
assert not any(line.rstrip().endswith(("?limit=2", "?limit=3", "?limit=4", "?limit=6")) for line in requests)
print("phenotype pages share one fixed provider order")
PY
```

Provider exhaustion and local continuation are independent. Early pages offer
the next buffered offset while warning about the ceiling; the last normalized
page retains the warning without inventing a continuation.

```bash
../../tools/biomcp-ci search phenotype 'HP:0001250 HP:0001263' --limit 2 | mustmatch like 'Continue with `--limit 2 --offset 2`
additional provider matches may exist beyond the 50-result window
Provider window: 50 raw rows received; limit 50; exhausted: true'
../../tools/biomcp-ci --json search phenotype 'HP:0001250 HP:0001263' --limit 2 | jq '.pagination.has_more and .pagination.provider_window_exhausted and (._meta.next_commands[0] | endswith("--limit 2 --offset 2"))' | mustmatch 'true'
../../tools/biomcp-ci search phenotype 'HP:0001250 HP:0001263' --limit 5 --offset 45 | mustmatch like 'Showing 3 results (total unknown).
additional provider matches may exist beyond the 50-result window
Provider window: 50 raw rows received; limit 50; exhausted: true'
../../tools/biomcp-ci --json search phenotype 'HP:0001250 HP:0001263' --limit 5 --offset 45 | jq '(.count == 3) and (.pagination.has_more == false) and .pagination.provider_window_exhausted and (._meta.next_commands | all(startswith("biomcp search phenotype ") | not))' | mustmatch 'true'
```

## Raw MCP Uses The Same Phenotype Contract

The raw escape hatch preserves the Markdown continuation and ceiling metadata,
and `json:true` returns the same structured pagination. Phenotype remains
absent from the typed search schema.

```bash
python3 - <<'PY' | mustmatch like 'raw MCP phenotype metadata agrees'
import json, os, subprocess

proc = subprocess.Popen([os.environ["BIOMCP_BIN"], "serve"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, env=os.environ.copy())
def call(message):
    proc.stdin.write(json.dumps(message) + "\n")
    proc.stdin.flush()
    return json.loads(proc.stdout.readline())

call({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"spec","version":"1"}}})
proc.stdin.write(json.dumps({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}) + "\n")
proc.stdin.flush()
default = call({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"biomcp","arguments":{"command":"biomcp search phenotype 'HP:0001250 HP:0001263' --limit 2"}}})["result"]
structured = call({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"biomcp","arguments":{"command":"biomcp search phenotype 'HP:0001250 HP:0001263' --limit 2","json":True}}})["result"]
assert default.get("isError") is False and structured.get("isError") is False
text = default["content"][0]["text"]
assert "--limit 2 --offset 2" in text and "Provider window: 50 raw rows received; limit 50; exhausted: true" in text
payload = json.loads(structured["content"][0]["text"])
assert payload["pagination"]["has_more"] is True
assert payload["pagination"]["provider_window_limit"] == 50
assert payload["pagination"]["provider_raw_row_count"] == 50
assert payload["pagination"]["provider_window_exhausted"] is True
proc.terminate(); proc.wait(timeout=5)

shell = open("../../src/mcp/shell.rs", encoding="utf-8").read()
typed_branches = shell.split("fn typed_search_schema", 1)[1].split("fn typed_variant_erepo_schema", 1)[0]
assert '"phenotype"' not in typed_branches
print("raw MCP phenotype metadata agrees")
PY
```

## Search bounds fail before provider contact

Phenotype queries accept at most ten unique HPO terms and only the first 50
ranked Monarch results. Unsupported windows are usage errors rather than
authoritative empty pages.

```bash
request_log="$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG"
invalid_window_path='/monarch/v3/api/semsim/search/HP:0001250/Human%20Diseases'
too_many_terms_path='/monarch/v3/api/semsim/search/HP:0000001,HP:0000002,HP:0000003,HP:0000004,HP:0000005,HP:0000006,HP:0000007,HP:0000008,HP:0000009,HP:0000010,HP:0000011/Human%20Diseases'
before_invalid_window="$(grep -Fc "$invalid_window_path" "$request_log" || true)"
before_too_many_terms="$(grep -Fc "$too_many_terms_path" "$request_log" || true)"
{ set +e; ../../tools/biomcp-ci --json search phenotype 'HP:0001250' --limit 11 --offset 40 2>/dev/null; status=$?; set -e; test "$status" -eq 2; } | mustmatch like '"code": "invalid_argument"
--offset + --limit must be <= 50 for phenotype search'
{ set +e; ../../tools/biomcp-ci --json search phenotype 'HP:0000001 HP:0000002 HP:0000003 HP:0000004 HP:0000005 HP:0000006 HP:0000007 HP:0000008 HP:0000009 HP:0000010 HP:0000011' 2>/dev/null; status=$?; set -e; test "$status" -eq 2; } | mustmatch like '"code": "invalid_argument"
at most 10 unique HPO terms'
after_invalid_window="$(grep -Fc "$invalid_window_path" "$request_log" || true)"
after_too_many_terms="$(grep -Fc "$too_many_terms_path" "$request_log" || true)"
test "$after_invalid_window" -eq "$before_invalid_window"
test "$after_too_many_terms" -eq "$before_too_many_terms"
```

## Observed Phenotype Provider Requests

The fixture fails closed outside the recorded HPO query and the exact Monarch term sets and limits.

```bash
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 >/dev/null
../../tools/biomcp-ci search phenotype 'HP:0001250 HP:0001263' --limit 2 >/dev/null
grep -F 'GET /hpo/search?q=seizure' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | mustmatch like 'search?q=seizure'
grep -F 'GET /monarch/v3/api/semsim/search/HP:0001250,HP:0001263/Human%20Diseases?limit=50' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | mustmatch like 'Diseases?limit=50'
base="$(cat "$BIOMCP_DISEASE_SURVIVAL_READY_FILE")"
for old_limit in 2 3 4 6; do
  status="$(curl -sS -o /dev/null -w '%{http_code}' "$base/monarch/v3/api/semsim/search/HP:0001250,HP:0001263/Human%20Diseases?limit=$old_limit")"
  test "$status" -eq 404
done
printf 'former limit-dependent phenotype routes rejected\n' | mustmatch like 'former limit-dependent phenotype routes rejected'
```
