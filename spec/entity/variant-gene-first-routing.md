# Variant Gene-Symbol First Free-Text Routing

A free-text `search variant "GENE condition"` phrase routes its first token to
the gene filter only when the token is an official gene symbol (tickets 1301
and 2022). Aliases do not route: the abbreviations HCC (HYCC1), MODY (HNF4A),
and HHT (ACVRL1) each resolve uniquely through their alias, so each keeps the
whole-phrase condition search. Symbol recognition uses MyGene's unique
entrez-backed official-symbol resolution behind the
`BIOMCP_VARIANT_QUERY_GENE_ROUTING` preference (`mygene` default, `off`
restores the whole-phrase condition search). A remainder that starts with a
protein change splits before routing, so `BRAF V600E melanoma` reads as gene,
protein change, and condition instead of `condition='V600E melanoma'`.
Refusal beats wrong routing: an uppercase non-gene first token such as BRUGADA
is refused, the whole phrase stays a condition search, and a zero-row refusal
prints the explicit `-g`/`--condition` working form. A routed search that
returns nothing prints the parsed form and the working alternative. These rows
replay recorded MyGene and MyVariant responses through the routine fixture
bases.

| phrase | count | filters | working_form | str:label |
|---|---|---|---|---|
| SCN5A Brugada | 3 | condition,gene | none | official symbol routes |
| BRUGADA syndrome | 0 | condition | biomcp search variant -g BRUGADA --condition syndrome | uppercase non-gene word refuses |
| brugada syndrome | 1 | condition | none | non-gene phrase keeps condition search |
| HCC liver cancer | 1 | condition | none | alias abbreviation refuses (HYCC1) |
| MODY diabetes | 1 | condition | none | alias abbreviation refuses (HNF4A) |
| HHT telangiectasias | 1 | condition | none | alias abbreviation refuses (ACVRL1) |
| BRAF V600E melanoma | 0 | condition,gene,hgvsp | biomcp search variant -g BRAF --hgvsp V600E --condition melanoma | protein change splits out of the condition |

```bash each_row="Variant Gene-Symbol First Free-Text Routing"
biomcp --json --no-cache search variant '{{phrase}}' --limit 3 \
  | jq -c '{count: .count, filters: (.filter_evaluation | keys | join(",")), working_form: (._meta.next_commands | map(select(startswith("biomcp search variant -g"))) | join(";") | if . == "" then "none" else . end)}' \
  | mustmatch like '{"count":{{count}},"filters":"{{filters}}","working_form":"{{working_form}}"}'
```

The routed phrase returns the same rows the explicit gene and condition flags
return, so a routed search never silently swaps the result set.

```bash
routed="$(biomcp --json --no-cache search variant 'SCN5A Brugada' --limit 3)"
explicit="$(biomcp --json --no-cache search variant -g SCN5A --condition Brugada --limit 3)"
jq -cn --argjson routed "$routed" --argjson explicit "$explicit" \
  '{routed_count: ($routed.count == 3), same_rows_as_explicit_form: ($routed.results == $explicit.results)}' \
  | mustmatch like '{"routed_count":true,"same_rows_as_explicit_form":true}'
```

Markdown states the routing in its query line, and a refused zero names the
working form instead of leaving the gene symbol swallowed.

```bash
biomcp --no-cache search variant 'SCN5A Brugada' --limit 3 \
  | grep -E '^Found:|^Query:' \
  | mustmatch like 'Found: 3 variant(s)
Query: gene=SCN5A, condition=Brugada'
```

```bash
biomcp --no-cache search variant 'BRUGADA syndrome' --limit 3 \
  | grep -E '^Query:|^No variants matched the phrase' \
  | mustmatch like 'Query: condition=BRUGADA syndrome
No variants matched the phrase as a condition. If BRUGADA is a gene symbol, try the working form: biomcp search variant -g BRUGADA --condition syndrome'
```

An alias abbreviation that resolves uniquely to one official symbol still
refuses, so the phrase stays one condition search.

```bash
biomcp --no-cache search variant 'HCC liver cancer' --limit 3 \
  | grep -E '^Query:' \
  | mustmatch like 'Query: condition=HCC liver cancer'
```

A routed phrase with a leading protein change prints the split parse, and a
routed zero states how the phrase was read beside the working alternative;
the alternative repeats the same filters, so the hint says to drop or loosen
one rather than promising rows.

```bash
biomcp --no-cache search variant 'BRAF V600E melanoma' --limit 3 \
  | grep -E '^Query:|^No variants matched' \
  | mustmatch like 'Query: gene=BRAF, hgvsp=V600E, condition=melanoma
No variants matched the routed phrase. Read as gene=BRAF, hgvsp=V600E, condition=melanoma. Drop or loosen a filter in the working form: biomcp search variant -g BRAF --hgvsp V600E --condition melanoma'
```
