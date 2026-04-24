# Variant Queries

Variant workflows need to balance exact identity with search-time normalization.
These canaries keep the stable column contracts, normalization rules, and
opt-in clinical sections without depending on brittle row counts.

## Gene-Scoped Variant Search

Gene-first search should still return the canonical variant identity columns and
preserve the BRAF V600E row as a recognizable anchor.

```bash
out="$(../../tools/biomcp-ci search variant -g BRAF --limit 3)"
echo "$out" | mustmatch like "| ID | Gene | Protein | Legacy Name |"
echo "$out" | mustmatch like "Query: gene=BRAF"
echo "$out" | mustmatch '/\| chr7:g\.\d+[ACGT]>[ACGT] \| BRAF \| p\.[A-Z*]\d+[A-Z*] \|/'
```

## Search Table Contract

The JSON path should keep the same follow-up shape so agents can pivot into the
default card without scraping markdown helper text.

```bash
json_out="$(../../tools/biomcp-ci --json search variant -g BRAF --limit 3)"
echo "$json_out" | mustmatch like '"next_commands":'
echo "$json_out" | jq -e '._meta.next_commands[0] | test("^biomcp get variant .+$")' >/dev/null
echo "$json_out" | jq -e '._meta.next_commands | any(. == "biomcp list variant")' >/dev/null
```

## Protein-Filter Narrowing

Long-form protein filters should normalize to the same compact spelling that the
short-form query uses, rather than leaking a second variant identifier shape.

```bash
out="$(../../tools/biomcp-ci search variant -g BRAF --hgvsp p.Val600Glu --limit 3)"
echo "$out" | mustmatch like "Query: gene=BRAF, hgvsp=V600E"
echo "$out" | mustmatch like "| chr7:g.140453136A>T | BRAF | p.V600E |"
```

## Residue-Alias Search

Residue aliases should stay on the typed variant path instead of falling
through to free-text or disease-style fallback behavior.

```bash
out="$(../../tools/biomcp-ci search variant 'PTPN22 620W' --limit 5)"
echo "$out" | mustmatch like "gene=PTPN22"
echo "$out" | mustmatch like "residue_alias=620W"
```

## Clinical Significance

ClinVar remains an opt-in deepen path. The section should keep the human heading
and a compact JSON disease anchor without bloating the default card.

```bash
out="$(../../tools/biomcp-ci get variant 'BRAF V600E' clinvar)"
echo "$out" | mustmatch like "## ClinVar"
echo "$out" | mustmatch like "Variant ID:"
```

```bash
json_out="$(../../tools/biomcp-ci --json get variant 'BRAF V600E' clinvar)"
echo "$json_out" | mustmatch like '"top_disease": {'
echo "$json_out" | jq -e '.top_disease.condition | type == "string"' >/dev/null
```

## Population Frequency

Population frequency also stays opt-in. The markdown and JSON views should keep
the same compact gnomAD frequency story.

```bash
out="$(../../tools/biomcp-ci get variant 'BRAF V600E' population)"
echo "$out" | mustmatch like "## Population"
echo "$out" | mustmatch '/gnomAD AF: .*%/'
```

## Variant Follow-Ups

The default card should still advertise typed follow-ups for downstream trial
and article pivots even when those surfaces are covered elsewhere.

```bash
json_out="$(../../tools/biomcp-ci --json get variant 'BRAF V600E')"
echo "$json_out" | mustmatch like '"next_commands": ['
echo "$json_out" | jq -e '._meta.next_commands | any(. | startswith("biomcp variant trials "))' >/dev/null
echo "$json_out" | jq -e '._meta.next_commands | any(. | startswith("biomcp variant articles "))' >/dev/null
```

## ID Normalization

Exact variant lookup should normalize equivalent identifiers back to the same
canonical record instead of splitting the user into parallel identities.

```bash
rsid="$(../../tools/biomcp-ci --json get variant rs113488022 | jq -r '.id')"
protein="$(../../tools/biomcp-ci --json get variant 'BRAF V600E' | jq -r '.id')"
test -n "$rsid"
printf '%s\n' "$rsid" | mustmatch "$protein"
printf '%s\n' "$rsid" | mustmatch '/^chr7:g\.\d+[ACGT]>[ACGT]$/'
```
