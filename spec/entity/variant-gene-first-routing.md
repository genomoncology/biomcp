# Variant Gene-Symbol First Free-Text Routing

A free-text `search variant "GENE condition"` phrase whose first token is a
known gene symbol routes that token to the gene filter and the remainder to
the condition (ticket 1301). Symbol recognition uses MyGene's unique
canonical symbol/alias resolution, the same lookup `discover` trusts, behind
the `BIOMCP_VARIANT_QUERY_GENE_ROUTING` preference (`mygene` default, `off`
restores the whole-phrase condition search). Refusal beats wrong routing: an
uppercase non-gene first token such as BRUGADA is refused, the whole phrase
stays a condition search, and a zero-row refusal prints the explicit
`-g`/`--condition` working form. These rows replay recorded MyGene and
MyVariant responses through the routine fixture bases.

| phrase | count | filters | working_form | str:label |
|---|---|---|---|---|
| SCN5A Brugada | 3 | condition,gene | none | known gene symbol routes |
| BRUGADA syndrome | 0 | condition | biomcp search variant -g BRUGADA --condition syndrome | uppercase non-gene word refuses |
| brugada syndrome | 1 | condition | none | non-gene phrase keeps condition search |

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
