# Gene Queries

Gene search is the fastest way to anchor a BioMCP session in a stable entity.
These canaries keep the focus on durable identity, deepen-path guidance, and
opt-in sections instead of volatile upstream counts or copy-edit trivia.

## Symbol-Based Search

Symbol search should still surface the canonical BRAF row in a human-scannable
table before the user pivots into deeper sections.

```bash
out="$(../../tools/biomcp-ci search gene BRAF --limit 3)"
echo "$out" | mustmatch like "# Genes: BRAF"
echo "$out" | mustmatch like "B-Raf proto-oncogene"
```

## Search Table Contract

The search surface needs to stay readable for humans and still expose machine
follow-ups through `_meta.next_commands`.

```bash
json_out="$(../../tools/biomcp-ci --json search gene BRAF --limit 3)"
echo "$json_out" | mustmatch like '"next_commands":'
echo "$json_out" | jq -e '._meta.next_commands[0] | test("^biomcp get gene .+$")' >/dev/null
echo "$json_out" | jq -e '._meta.next_commands | any(. == "biomcp list gene")' >/dev/null
```

## Identity Card

The default card should keep the persistent identifier and the progressive
disclosure hints that let readers deepen into the right follow-up section.

```bash
out="$(../../tools/biomcp-ci get gene BRAF)"
echo "$out" | mustmatch like "Entrez ID: 673"
echo "$out" | mustmatch like "biomcp get gene BRAF pathways"
echo "$out" | mustmatch like "biomcp get gene BRAF diagnostics"
```

## All-Section Warm Budget

The combined gene bundle should stay fast on a warm spec cache while still
rendering deep content, not just the identity card.

```bash
../../tools/biomcp-ci get gene BRCA1 all >/dev/null
start_ns="$(date +%s%N)"
out="$(../../tools/biomcp-ci get gene BRCA1 all)"
end_ns="$(date +%s%N)"
elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
if [ "$elapsed_ms" -ge 7000 ]; then
  echo "expected warm biomcp get gene BRCA1 all under 7000ms, got ${elapsed_ms}ms" >&2
  exit 1
fi
echo "$out" | mustmatch like "## ClinGen"
```

## Tissue-Expression Context

Human Protein Atlas data belongs in an opt-in deepen path. When live HPA data is
missing, BioMCP should stay truthful rather than fabricating tissue rows.

```bash
out="$(../../tools/biomcp-ci get gene BRAF hpa)"
echo "$out" | mustmatch like "## Human Protein Atlas"
if printf '%s\n' "$out" | grep -q 'No Human Protein Atlas records returned'; then
  echo "$out" | mustmatch like "No Human Protein Atlas records returned"
else
  echo "$out" | mustmatch like "Reliability:"
  echo "$out" | mustmatch like "| Tissue | Level |"
  echo "$out" | mustmatch like "Subcellular"
fi
```

## Druggability & Targets

Targetability context should stay separate from the default card while still
showing the combined OpenTargets and DGIdb story for actionable genes.

```bash
out="$(../../tools/biomcp-ci get gene EGFR druggability)"
echo "$out" | mustmatch like "## Druggability"
echo "$out" | mustmatch like "OpenTargets tractability"
echo "$out" | mustmatch like "| antibody | yes | Approved Drug"
```

## Funding & Diagnostics Cross-Pivot

Funding remains opt-in, but the base gene view still needs to advertise the
diagnostics deepen path so operators can move from one card into the next.

```bash
funding="$(../../tools/biomcp-ci get gene ERBB2 funding)"
echo "$funding" | mustmatch like "## Funding (NIH Reporter)"
echo "$funding" | mustmatch like "| Project | PI | Organization | FY | Amount |"
```

```bash
json_out="$(../../tools/biomcp-ci --json get gene BRCA1)"
echo "$json_out" | jq -e '._meta.next_commands | any(. == "biomcp get gene BRCA1 diagnostics")' >/dev/null
echo "$json_out" | jq -e '._meta.next_commands | any(. == "biomcp get gene BRCA1 pathways")' >/dev/null
```
