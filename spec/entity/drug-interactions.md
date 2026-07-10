# Bounded DDInter interaction reads

DDInter interaction detail is read from an installed local bundle and returned
in stable, explicit pages. These fixture-backed examples document paging and
freshness without contacting DDInter or simulating a failed service.

## Discover interaction page controls

The helper advertises the same page controls that scripts and agents use,
including both limit and offset controls. The examples below prove the bounded
25-row default without pinning help copy.

```bash
../../tools/biomcp-ci drug interactions --help | mustmatch like '--limit <LIMIT>
--offset <OFFSET>'
```

## Page through interaction detail

BioMCP sorts the complete local match set before taking a page. Page metadata
makes the bounded response explicit and gives the exact command for continuing
without silently dropping the remaining DDInter rows.

```bash
../../tools/biomcp-ci --json drug interactions warfarin | jq -c '{pagination, bundle_freshness, first: .interactions[0].drug, last: .interactions[-1].drug}' | mustmatch like '{"pagination":{"total":27,"count":25,"offset":0,"limit":25,"next_command":"biomcp drug interactions warfarin --limit 25 --offset 25"},"bundle_freshness":{"status":"fresh"},"first":"Amoxicillin","last":"Partner 23"}'
```

The final page continues at the advertised offset, preserves the established
severity-and-name order, and does not offer a dead continuation command.

```bash
../../tools/biomcp-ci --json drug interactions warfarin --limit 25 --offset 25 | jq -c '{pagination, first: .interactions[0].drug, last: .interactions[-1].drug, next_command: (.pagination.next_command // null)}' | mustmatch like '{"pagination":{"total":27,"count":2,"offset":25,"limit":25},"first":"Partner 24","last":"Atorvastatin","next_command":null}'
```

## Bound the standard drug section

`get drug <name> interactions` provides the same bounded first page and directs
operators to the page-able helper rather than hiding an unbounded alternate
path.

```bash
../../tools/biomcp-ci get drug warfarin interactions | mustmatch like '## Interactions (DDInter)
Returned: 25 of 27
DDInter bundle freshness: fresh
biomcp drug interactions warfarin --limit 25 --offset 25'
```
