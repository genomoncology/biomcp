# Alias Fallback

`get gene` and `get drug` now use the discovery layer on exact-match misses to
offer canonical retry guidance without silently rewriting the request. These
checks focus on stable recovery markers, non-success exit behavior, and JSON/MCP
retry metadata.

| Section | Command focus | Why it matters |
|---|---|---|
| Gene Alias Fallback | `get gene ERBB1` / `P53` / `HER2` | Confirms alias miss produces recovery guidance pointing to canonical symbol |
| Drug Brand Passthrough | `get drug Keytruda` / `Herceptin` | Confirms brand-name drugs are found directly (MyChem handles brand names) |
| Ambiguous Miss | `get gene V600E` | Confirms ambiguous misses point to `discover` |
| Canonical Passthrough | `get gene TP53` / `get drug imatinib` | Confirms exact canonical lookups still succeed directly |
| JSON Metadata Contract | `--json get gene ERBB1` | Confirms `_meta.alias_resolution` + `_meta.next_commands` on exit 1 |

## Gene Alias Fallback

OLS4 returns multiple candidates for common gene aliases, so the confidence gate
produces ambiguous guidance rather than a single canonical rewrite. The canonical
symbol appears in the candidate list and the output directs users to `discover`
for disambiguation.

```bash
status=0
out="$(biomcp get gene ERBB1 2>&1)" || status=$?
test "${status}" -eq 1
echo "$out" | mustmatch like "BioMCP could not map 'ERBB1' to a single gene."
echo "$out" | mustmatch like "biomcp discover ERBB1"
echo "$out" | mustmatch like "EGFR"

status=0
out="$(biomcp get gene P53 2>&1)" || status=$?
test "${status}" -eq 1
echo "$out" | mustmatch like "BioMCP could not map 'P53' to a single gene."
echo "$out" | mustmatch like "biomcp discover P53"
echo "$out" | mustmatch like "TP53"

status=0
out="$(biomcp get gene HER2 2>&1)" || status=$?
test "${status}" -eq 1
echo "$out" | mustmatch like "BioMCP could not map 'HER2' to a single gene."
echo "$out" | mustmatch like "biomcp discover HER2"
echo "$out" | mustmatch like "ERBB2"
```

## Drug Brand Passthrough

MyChem.info resolves brand names directly, so `get drug Keytruda` and
`get drug Herceptin` succeed without invoking the alias fallback path.

```bash
out="$(biomcp get drug Keytruda)"
echo "$out" | mustmatch like "# keytruda"
echo "$out" | mustmatch not like "Did you mean:"

out="$(biomcp get drug Herceptin)"
echo "$out" | mustmatch like "# herceptin"
echo "$out" | mustmatch not like "Did you mean:"
```

## Ambiguous Miss

```bash
status=0
out="$(biomcp get gene V600E 2>&1)" || status=$?
test "${status}" -eq 1
echo "$out" | mustmatch like "BioMCP could not map 'V600E' to a single gene."
echo "$out" | mustmatch like "biomcp discover V600E"
echo "$out" | mustmatch like "biomcp search gene -q V600E"
```

## Canonical Passthrough

```bash
out="$(biomcp get gene TP53)"
echo "$out" | mustmatch like "# TP53"
echo "$out" | mustmatch not like "Did you mean:"

out="$(biomcp get drug imatinib)"
echo "$out" | mustmatch like "# imatinib"
echo "$out" | mustmatch not like "Did you mean:"
```

## JSON Metadata Contract

ERBB1 produces an ambiguous alias result (multiple OLS4 candidates). The JSON
payload confirms the structured metadata contract: kind, candidates containing
the canonical symbol, and ordered next_commands.

```bash
status=0
out="$(biomcp --json get gene ERBB1)" || status=$?
test "${status}" -eq 1
echo "$out" | mustmatch like '"alias_resolution": {'
echo "$out" | mustmatch like '"kind": "ambiguous"'
echo "$out" | mustmatch like '"EGFR"'
echo "$out" | mustmatch like '"next_commands": ['
echo "$out" | mustmatch like '"biomcp discover ERBB1"'
```
