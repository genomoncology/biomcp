# CTGov Helper Pivot Bounded Requests

Gene and disease trial pivots are quick entry points from an entity card to the
trial search surface. When a user asks for only one CTGov result, BioMCP should
return that result without asking CTGov for avoidable total-count work.

## Limit-one helper pivots avoid CTGov total-count work

A SHANK3 gene trial pivot still returns the first trial result through the CTGov
search path. This fixture-backed smoke keeps the command and result shape visible
to users. The native `gene_trial_filters_keep_single_ctgov_result_countless` and
`disease_trial_filters_keep_single_ctgov_result_countless` tests own literal
query fields and count-suppression request construction.

```bash
../../spec/fixtures/ctgov-request-log run ../../tools/biomcp-ci --json gene trials SHANK3 --limit 1 \
  | jq -r '.count, (.results[0].nct_id | startswith("NCT"))' \
  | mustmatch like '1
true'
```
