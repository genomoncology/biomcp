# Phenotype Queries

Phenotype search turns symptom language or HPO IDs into a ranked disease shortlist. These captured contracts use the shipped CLI against fresh HPO and Monarch responses served by the supervised routine fixture.

## Captured Symptom-Phrase Route

The routine fixture resolves the symptom phrase through HPO search and then replays the exact Monarch similarity request produced by those identifiers.

```bash
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 | mustmatch like '# Phenotype Search: seizure, developmental delay
Resolved HPO terms:
`HP:0001250` — Seizure
`HP:0001263` — Global developmental delay
Semantic Similarity Score
MONDO:0010450
supported'
```

## Captured HPO-ID Route

Direct HPO IDs skip phrase resolution and use the same similarity and rendering path.

```bash
../../tools/biomcp-ci search phenotype 'HP:0001250 HP:0001263' --limit 3 | mustmatch like '# Phenotype Search: HP:0001250 HP:0001263
Resolved HPO terms:
`HP:0001250` — Seizure
`HP:0001263` — Global developmental delay
Semantic Similarity Score
MONDO:0010450
intellectual disability, X-linked 89'
```

## Disease Follow-Up Guidance

The captured phrase result teaches the typed disease command only because its
first returned row has exact support for every resolved term.

```bash
../../tools/biomcp-ci search phenotype 'seizure, developmental delay' --limit 3 | mustmatch like 'See also:
biomcp get disease MONDO:0010450 phenotypes'
```

## Captured JSON Follow-Up Envelope

JSON callers receive the same typed disease follow-up without an unsupported phenotype-get command.

```bash
../../tools/biomcp-ci --json search phenotype 'HP:0001250 HP:0001263' --limit 1 | jq '(.resolved_query == [{"raw":"HP:0001250","id":"HP:0001250","label":"Seizure"},{"raw":"HP:0001263","id":"HP:0001263","label":"Global developmental delay"}]) and (.results[0].direct_support | all(.status == "supported")) and (.pagination.total == null) and .pagination.has_more and (.pagination.provider_window_limit == 50) and (.pagination.provider_raw_row_count == 50) and .pagination.provider_window_exhausted and (._meta.next_commands | any(startswith("biomcp search phenotype ") and endswith("--limit 1 --offset 1"))) and (._meta.next_commands | any(. == "biomcp get disease MONDO:0010450 phenotypes"))' | mustmatch 'true'
```

Both renderers use one ordered command selector: continuation first, then the
first fully supported disease, then the existing list helper.

```bash
../../tools/biomcp-ci --json search phenotype 'HP:0001250 HP:0001263' --limit 1 | jq -c '._meta.next_commands | [(.[0] | endswith("--limit 1 --offset 1")), .[1], .[2]]' | mustmatch '[true,"biomcp get disease MONDO:0010450 phenotypes","biomcp list phenotype"]'
../../tools/biomcp-ci search phenotype 'HP:0001250 HP:0001263' --limit 1 | mustmatch like 'See also:
biomcp search phenotype
biomcp get disease MONDO:0010450 phenotypes
biomcp list phenotype'
../../tools/biomcp-ci --json search phenotype macrocephaly --limit 1 | jq -c '._meta.next_commands | [(.[0] | endswith("--limit 1 --offset 1")), .[1]]' | mustmatch '[true,"biomcp list phenotype"]'
../../tools/biomcp-ci search phenotype macrocephaly --limit 1 | mustmatch like 'See also:
biomcp search phenotype macrocephaly
biomcp list phenotype'
```

Phrase and direct-ID inputs resolve to the same fixed provider window.

```bash
../../tools/biomcp-ci --json search phenotype 'seizure, developmental delay' --limit 3 | jq '(.resolved_query | map(.id)) == ["HP:0001250","HP:0001263"] and (.pagination.provider_window_limit == 50) and (.pagination.provider_raw_row_count == 50) and .pagination.provider_window_exhausted' | mustmatch 'true'
```

## Similarity Does Not Imply Direct Support

The adversarial macrocephaly response ranks isolated microcephaly first. Its
complete direct lookup does not contain the requested pair, while the second
candidate has exact support and alone receives the follow-up.

```bash
../../tools/biomcp-ci search phenotype macrocephaly --limit 2 | mustmatch like 'semantic-similarity candidate
MONDO:0019387 | isolated microcephaly | 19.000 | `HP:0000256`: not_supported
no direct Monarch association was found in the complete lookup
biomcp get disease MONDO:0001234 phenotypes'
../../tools/biomcp-ci --json search phenotype macrocephaly --limit 2 | jq '(.resolved_query == [{"raw":"macrocephaly","id":"HP:0000256","label":"Macrocephaly"}]) and (.results | map(.direct_support[0].status)) == ["not_supported","supported"] and (._meta.next_commands | any(. == "biomcp get disease MONDO:0001234 phenotypes")) and (._meta.next_commands | any(contains("MONDO:0019387")) | not)' | mustmatch 'true'
```

Every free-text phrase must contribute at least one HPO row. A successful
empty array remains a user-input error and prevents both Monarch calls.

```bash
python3 - <<'PY' | mustmatch like 'unresolved phrase rejected before Monarch'
import os, subprocess
log = os.environ["BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG"]
before = open(log, encoding="utf-8").read().count("/monarch/")
result = subprocess.run([os.environ["BIOMCP_BIN"], "--json", "search", "phenotype", "macrocephaly, phrase-with-no-hpo-row"], capture_output=True, text=True, env=os.environ.copy())
assert result.returncode == 2 and "phrase-with-no-hpo-row" in result.stdout
after = open(log, encoding="utf-8").read().count("/monarch/")
assert after == before
print("unresolved phrase rejected before Monarch")
PY
```

Missing completeness fields never become negative evidence, while an
association outage degrades only the support phase.

```bash
../../tools/biomcp-ci --json search phenotype HP:0000201 --limit 1 | jq '(.results[0].score == 7) and (.results[0].direct_support == [{"hpo_id":"HP:0000201","status":"indeterminate"}]) and (._meta.next_commands | all(startswith("biomcp get disease ") | not))' | mustmatch 'true'
../../tools/biomcp-ci search phenotype HP:0000202 --limit 1 | mustmatch like 'support state fixture disease
`HP:0000202`: unavailable
direct-support enrichment failed
No disease follow-up is suggested'
../../tools/biomcp-ci --json search phenotype HP:0000206 --limit 1 | jq -r '.results[0].direct_support[0].status' | mustmatch 'indeterminate'
../../tools/biomcp-ci --json search phenotype HP:0000207 --limit 1 | jq -r '.results[0].direct_support[0].status' | mustmatch 'indeterminate'
../../tools/biomcp-ci --json search phenotype HP:0000208 --limit 1 | jq -r '.results[0].direct_support[0].status' | mustmatch 'not_supported'
../../tools/biomcp-ci --json search phenotype HP:0000209 --limit 2 | jq -c '.results | map(.direct_support[0].status)' | mustmatch '["supported","indeterminate"]'
../../tools/biomcp-ci --json search phenotype HP:0000210 --limit 2 | jq -c '.results | map(.direct_support[0].status)' | mustmatch '["supported","indeterminate"]'
../../tools/biomcp-ci --json search phenotype 'HP:0000209 HP:0000210' --limit 2 | jq -c '{states: [.results[] | [.direct_support[].status]], follow_up: (._meta.next_commands | map(select(startswith("biomcp get disease "))))}' | mustmatch '{"states":[["supported","not_supported"],["supported","supported"]],"follow_up":["biomcp get disease MONDO:0000201 phenotypes"]}'
```

Malformed HPO search envelopes are provider failures, including when a sibling
phrase returns a valid empty array. A valid empty array remains the distinct
unresolved-input outcome, and no case reaches Monarch.

```bash
python3 - <<'PY' | mustmatch like 'HPO envelope precedence is fail closed'
import os, subprocess
log = os.environ["BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG"]
binary = os.environ["BIOMCP_BIN"]
for query, malformed in [
    ("missing-terms", True), ("null-terms", True),
    ("scalar-terms", True), ("empty-control", False),
    ("empty-control, missing-terms", True),
]:
    before = open(log, encoding="utf-8").read().count("/monarch/")
    result = subprocess.run([binary, "--json", "search", "phenotype", query], capture_output=True, text=True, env=os.environ.copy())
    assert result.returncode != 0
    assert ("No HPO terms matched" not in result.stdout) == malformed, (query, result.stdout)
    assert open(log, encoding="utf-8").read().count("/monarch/") == before
print("HPO envelope precedence is fail closed")
PY
```

Aggregate free-text resolution is all-or-nothing: ten unique IDs preserve the
first phrase for a duplicate, while the eleventh is rejected before Monarch.

```bash
python3 - <<'PY' | mustmatch like 'aggregate ten accepted and eleven rejected'
import json, os, subprocess
from urllib.parse import parse_qs, urlparse
binary = os.environ["BIOMCP_BIN"]
log = os.environ["BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG"]
ok = subprocess.run([binary, "--json", "search", "phenotype", "ten-first, ten-second", "--limit", "1"], check=True, capture_output=True, text=True, env=os.environ.copy())
payload = json.loads(ok.stdout)
assert [row["id"] for row in payload["resolved_query"]] == [f"HP:{index:07d}" for index in range(1, 11)]
assert payload["resolved_query"][4]["raw"] == "ten-first"
before = open(log, encoding="utf-8").read().count("/monarch/")
bad = subprocess.run([binary, "--json", "search", "phenotype", "eleven-first, eleven-second"], capture_output=True, text=True, env=os.environ.copy())
assert bad.returncode == 2 and "resolved more than 10 unique HPO terms" in bad.stdout
assert open(log, encoding="utf-8").read().count("/monarch/") == before
print("aggregate ten accepted and eleven rejected")
PY
```

Direct-ID labels are HPO-owned and fail closed before Monarch when the term
response is mismatched, blank, or absent.

```bash
python3 - <<'PY' | mustmatch like 'invalid HPO labels prevented Monarch contact'
import os, subprocess
log = os.environ["BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG"]
for term in ("HP:0000203", "HP:0000204", "HP:0000205"):
    before = open(log, encoding="utf-8").read().count("/monarch/")
    result = subprocess.run([os.environ["BIOMCP_BIN"], "--json", "search", "phenotype", term], capture_output=True, text=True, env=os.environ.copy())
    assert result.returncode != 0
    assert open(log, encoding="utf-8").read().count("/monarch/") == before
print("invalid HPO labels prevented Monarch contact")
PY
```

## Stable Provider Order And Local Pages

Every supported page is sliced from the same provider-ordered, normalized
window. The fixture includes tied scores, a later duplicate with a larger
score, and a non-disease row so sorting or counting after normalization cannot
accidentally satisfy the contract.

```bash
python3 - <<'PY' | mustmatch like 'phenotype pages share one fixed provider order'
import json, os, subprocess
from urllib.parse import parse_qs, urlparse

binary = os.environ["BIOMCP_BIN"]
env = os.environ.copy()
log = os.environ["BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG"]
initial_lines = open(log, encoding="utf-8").readlines()

def search(limit, offset=0):
    before_lines = open(log, encoding="utf-8").readlines()
    result = subprocess.run(
        [binary, "--json", "search", "phenotype", "HP:0001250 HP:0001263", "--limit", str(limit), "--offset", str(offset)],
        check=True, capture_output=True, text=True, env=env,
    )
    payload = json.loads(result.stdout)
    association = [line for line in open(log, encoding="utf-8").readlines()[len(before_lines):] if "/monarch/v3/api/association?" in line]
    expected_subjects = [row["disease_id"] for row in payload["results"]]
    assert len(association) == (1 if expected_subjects else 0)
    if association:
        request = parse_qs(urlparse(association[0].split(" ", 1)[1].strip()).query)
        assert request["subject"] == expected_subjects
        assert request["object"] == ["HP:0001250", "HP:0001263"]
    return payload

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

association_lines = [line for line in open(log, encoding="utf-8").readlines()[len(initial_lines):] if "/monarch/v3/api/association?" in line]
assert association_lines
for line in association_lines:
    parsed = urlparse(line.split(" ", 1)[1].strip())
    query = parse_qs(parsed.query)
    assert query["limit"] == ["500"] and query["offset"] == ["0"] and query["direct"] == ["true"]
    assert len(query["subject"]) in (1, 2, 3, 5)
    assert len(query["object"]) == 2

before_empty = open(log, encoding="utf-8").read().count("/monarch/v3/api/association?")
empty = search(2, 48)
assert empty["results"] == []
assert open(log, encoding="utf-8").read().count("/monarch/v3/api/association?") == before_empty

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
degraded = call({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"biomcp","arguments":{"command":"biomcp search phenotype HP:0000202 --limit 1","json":True}}})["result"]
assert default.get("isError") is False and structured.get("isError") is False
text = default["content"][0]["text"]
assert "--limit 2 --offset 2" in text and "Provider window: 50 raw rows received; limit 50; exhausted: true" in text
payload = json.loads(structured["content"][0]["text"])
assert payload["pagination"]["has_more"] is True
assert payload["pagination"]["provider_window_limit"] == 50
assert payload["pagination"]["provider_raw_row_count"] == 50
assert payload["pagination"]["provider_window_exhausted"] is True
degraded_payload = json.loads(degraded["content"][0]["text"])
assert degraded.get("isError") is False
assert degraded_payload["results"][0]["direct_support"][0]["status"] == "unavailable"
assert not any(command.startswith("biomcp get disease ") for command in degraded_payload["_meta"]["next_commands"])
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
before_hpo_searches="$(grep -Fc '/hpo/search?' "$request_log" || true)"
{ set +e; ../../tools/biomcp-ci --json search phenotype 'HP:0001250' --limit 11 --offset 40 2>/dev/null; status=$?; set -e; test "$status" -eq 2; } | mustmatch like '"code": "invalid_argument"
--offset + --limit must be <= 50 for phenotype search'
{ set +e; ../../tools/biomcp-ci --json search phenotype 'HP:0000001 HP:0000002 HP:0000003 HP:0000004 HP:0000005 HP:0000006 HP:0000007 HP:0000008 HP:0000009 HP:0000010 HP:0000011' 2>/dev/null; status=$?; set -e; test "$status" -eq 2; } | mustmatch like '"code": "invalid_argument"
at most 10 unique HPO terms'
{ set +e; ../../tools/biomcp-ci --json search phenotype 'p1,p2,p3,p4,p5,p6,p7,p8,p9,p10,p11' 2>/dev/null; status=$?; set -e; test "$status" -eq 2; } | mustmatch like '"code": "invalid_argument"
at most 10 comma-delimited symptom phrases'
after_invalid_window="$(grep -Fc "$invalid_window_path" "$request_log" || true)"
after_too_many_terms="$(grep -Fc "$too_many_terms_path" "$request_log" || true)"
test "$after_invalid_window" -eq "$before_invalid_window"
test "$after_too_many_terms" -eq "$before_too_many_terms"
test "$(grep -Fc '/hpo/search?' "$request_log" || true)" -eq "$before_hpo_searches"
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
