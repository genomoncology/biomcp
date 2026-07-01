# CTGov Helper Pivot Bounded Requests

Gene and disease trial pivots are quick entry points from an entity card to the
trial search surface. When a user asks for only one CTGov result, BioMCP should
return that result without asking CTGov for avoidable total-count work.

## Limit-one helper pivots avoid CTGov total-count work

A SHANK3 gene trial pivot still returns the first trial result through the CTGov
search path, but the request sent for that one row should not include the CTGov
`countTotal` option.

```bash
../../spec/fixtures/ctgov-request-log run ../../tools/biomcp-ci --json gene trials SHANK3 --limit 1 \
  | jq -r '.count, (.results[0].nct_id | startswith("NCT"))' \
  | mustmatch like '1
true'
```

```bash
../../spec/fixtures/ctgov-request-log show | mustmatch not like 'countTotal=true'
```

The same bounded request rule applies when a rare disease name is used directly.
The disease pivot should return the first fixture trial without turning the page
request into a total-count query.

```bash
../../spec/fixtures/ctgov-request-log run ../../tools/biomcp-ci --json disease trials "Phelan-McDermid Syndrome" --limit 1 \
  | jq -r '.count, (.results[0].nct_id | startswith("NCT"))' \
  | mustmatch like '1
true'
```

```bash
../../spec/fixtures/ctgov-request-log show | mustmatch not like 'countTotal=true'
```
