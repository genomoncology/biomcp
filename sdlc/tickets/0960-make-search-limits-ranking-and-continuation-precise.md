---
flow: build
priority: 7
deps: ["0957"]
---
# Make search limits, ranking, and continuation precise

Close three related search-presentation gaps without inventing a global
cross-provider rank: diagnostic JSON expands large nested arrays, multi-region
drug limits look global but are applied per region, and author rows with the
same display name are indistinguishable in human output.

## Diagnostic contract

Add `--full` to diagnostic search. Default JSON includes at most five genes and
five conditions per result, with exact total counts and `has_more` for each
list, matching the existing compact Markdown intent. `--full` returns the
complete arrays already present in the bounded provider result; it does not
widen the result count or ticket 0924's expanded-response limits. Remove the
duplicate source default from help so Clap states it exactly once.

## Drug contract

Keep the current `--limit` and `--offset`, but define both as per-region. An
all-region request can therefore return at most three times the limit. Help,
JSON, and Markdown say this directly. Each US/EU/WHO bucket reports its region,
offset, limit, returned count, provider total when known, `has_more`, and a
continuation command that selects only that region.

Within a region, use this stable order: exact normalized product name, exact
active substance, known alias, then broad text mention. Emit
`match_kind: product_name|active_substance|alias|broad_text`; preserve provider
order inside one kind. `alias` requires a structured alias supplied by the
existing typed identity resolver or a provider alias field; fuzzy, substring,
and free-text mentions remain `broad_text`. A broad EU textual mention can
remain visible but never outranks an exact pembrolizumab identity. Do not merge
or compare regulatory rank across regions. Within each region, deduplication,
match classification, and the complete four-tier ordering happen before the
user's per-region offset and limit are applied. A source adapter may use bounded
internal pages or tier-specific requests, but it may not page first and sort
only that slice.

## Author contract

Human author rows show the stable provider author ID, the first nonempty
affiliation shortened to 120 UTF-8 bytes, and paper, citation, and h-index
counts when present. Missing facts render as unknown, not zero. The header
reports provider total when known, offset, returned, and `has_more`; a next
command uses the provider-supplied continuation. JSON values and existing
evidence identities remain unchanged.

## Done when

- Recorded GTR rows with more than five genes/conditions prove compact/default
  counts and explicit full expansion in both renderers.
- Recorded US/EU/WHO fixtures prove per-region limits and offsets, unknown
  totals, exact ranking tiers, stable ties, and region-specific continuation.
  One fixture puts broad rows on an earlier provider page and an exact row on a
  later page, then proves user paging still places the exact row first.
- Two same-name author fixtures prove visible disambiguation, missing metrics,
  multibyte affiliation shortening, exact pagination, and no invented ORCID.
- Generated CLI/MCP help and user docs use these contracts consistently.

## Authorized test changes

Design commits may restate diagnostic, drug, and author search arguments,
typed presentation metadata, ranking, renderers, fixtures, schemas, and docs.
Provider query meaning outside the named ranking tiers stays unchanged.

The src line ceiling may rise by at most 260 lines.
