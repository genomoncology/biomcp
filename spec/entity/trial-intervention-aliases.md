# Trial Intervention Aliases

Intervention search can broaden a known drug name through useful identity aliases
without letting upstream synonym punctuation change ClinicalTrials.gov grammar.
The requested name remains authoritative while trade names add relevant trials.

## Literal, trial-safe intervention aliases

ClinicalTrials.gov intervention searches treat the requested drug and useful
trade names as literal names. Systematic chemical synonyms from identity data
must not turn one valid drug search into invalid CTGov query grammar.

```bash
../../spec/fixtures/ctgov-request-log run-with-mychem ../../tools/biomcp-ci --json search trial --intervention venetoclax --source ctgov --limit 5 \
  | jq -r '.results[] | "\(.nct_id): \(.matched_intervention_label)"' \
  | mustmatch 'NCT51000001: venetoclax
NCT51000002: Venclexta'
```

The requested name and useful trade alias are sent as quoted literal queries.
A separate plausible alias in the fixture is also sent, rejected by the CTGov
parser, and cannot discard these successful results.

```bash
../../spec/fixtures/ctgov-request-log show-interventions \
  | mustmatch like '"venetoclax"
"Parser Trap"
"Venclexta"'
```

Because a rejected expanded alias leaves coverage incomplete, the response does
not claim an exact total.

```bash
../../spec/fixtures/ctgov-request-log run-with-mychem ../../tools/biomcp-ci --json search trial --intervention venetoclax --source ctgov --limit 5 \
  | jq '.pagination | {total}' \
  | mustmatch like '{"total":null}'
```

Systematic source synonyms do not become additional trial-search workers.

```bash
../../spec/fixtures/ctgov-request-log show-interventions \
  | mustmatch not like 'benzoic acid
free base'
```

## Alias fanout continues after detail-backed rejection

A page that adds no eligible trial is not the end of an alias-expanded search.
When detail eligibility rejects every new study in the first fanout round, a
later page can still supply a qualifying trial for the requested intervention.

```bash
../../spec/fixtures/ctgov-request-log run-with-mychem ../../tools/biomcp-ci --json search trial --intervention venetoclax --criteria nextpageproof --source ctgov --limit 1 \
  | jq -r '.results[] | "\(.nct_id): \(.matched_intervention_label)"' \
  | mustmatch 'NCT51000004: Venclexta'
```
