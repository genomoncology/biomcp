---
flow: build
priority: 7
---

# Remove unrelated EMA products from exact drug search

## Goal

A name search must not turn an untyped chemical synonym into hundreds of EMA
matches through a generic word. On 2026-09-04,
`biomcp --json search drug eflornithine --region all --limit 2` reported 279 EU
matches. `Vaniqa` was relevant, but `Prasugrel Viatris` ranked second even
though its EMA name, active substance, and indication contain no
`eflornithine`. The captured evidence in commit `f8ff2a78` traces that false
positive to the token `acid` from DrugBank's systematic eflornithine synonym.

This is a search-identity defect, not a reason to redefine the public
`Drug.brand_names` field. Build a typed identity specifically for EMA search,
retain the useful primary-query and CDC vaccine behavior, and report the term
that actually admitted each EU row.

## Current facts

- `transform::drug::merge_mychem_hits` puts up to three arbitrary
  `drugbank.synonyms` values in `Drug.brand_names`.
- `build_ema_identity` currently passes that display field to
  `EmaDrugIdentity::with_aliases`; `search_tokens` then splits every term and
  accepts any token of three or more characters in an EMA active substance or
  indication.
- Record 0960 deliberately kept a primary drug-name phrase as a `broad_text`
  match in an EMA name, active substance, or indication. This ticket must keep
  that behavior. It removes token expansion from untyped aliases; it does not
  turn name search into exact-only search.
- The typed MCP `search` schema does not publish the `drug` entity. Drug search
  is exposed by the CLI and the read-only raw `biomcp` MCP command only.

## Search identity

Keep `Drug`, `merge_mychem_hits`, `Drug.brand_names`, `build_who_identity`, and
all `get drug` paths unchanged. For EMA name search, construct an internal
typed identity directly from the MyChem response; do not call or inherit
`select_hits_for_name` because that shared helper deliberately falls back to
all returned hits. Clean a candidate field by trimming, trimming leading and
trailing periods, collapsing whitespace, and comparing ASCII
case-insensitively, exactly as the current search-name normalizer does.
Admit a MyChem hit only when at least one allowed field value equals the
cleaned requested query. Inspect every value in `StringOrVec` and every NDC
row, rather than only its first value. After admission, only these fields from
that admitted hit may contribute terms:

| Term role | Allowed source field | Matching authority |
| --- | --- | --- |
| requested | the caller's query | exact identity and the existing boundary-phrase broad search |
| generic name | `openfda.generic_name`, `ndc.nonproprietaryname`, `drugbank.name`, or `chembl.pref_name` | exact EMA name/active-substance alias only |
| verified brand | `openfda.brand_name` | exact EMA name/active-substance alias only |
| vaccine bridge | the `cvx_short_description` or `cvx_full_vaccine_name` joined from a matching CDC CVX trade-name row | the bounded CVX rule below |

`drugbank.synonyms`, `gtopdb.name`, `unii.display_name`, and `chebi.name` are
neither hit-admission evidence nor EMA-search terms. In particular, no value
from `Drug.brand_names` is fed back into this identity. An irrelevant nonempty
response, or a response whose query match occurs only in an excluded field,
is unresolved; never restore the shared helper's all-hits fallback. Terms from
admitted hits are nonblank, normalized, case-insensitively deduplicated first
by provider-hit order and then by this explicit field order:
`openfda.generic_name` values, NDC rows' `nonproprietaryname`, `drugbank.name`,
`chembl.pref_name`, and `openfda.brand_name` values. They retain both their
original cleaned text and source. A MyChem error, empty response, or response
with zero admitted hits preserves the query-only EMA fallback and triggers the
existing CVX branch; it does not fail an otherwise usable local EMA search. Fixtures cover
all four cases and prove that excluded-only and irrelevant hits neither leak
terms nor suppress CVX.

Keep the existing CVX trigger: consult it for EU search only when MyChem did
not resolve the query (and for the already separate explicit WHO-vaccine
path). A CVX row qualifies only through the current exact normalized trade-name
or trade-name family-prefix match, so `gardasil` may select `Gardasil` and
`Gardasil 9`, `prevnar` may select `PREVNAR 13`, and `fluzone` may select its
named products. An arbitrary query must not search all CVX descriptions.

For each qualified CVX description, preserve the original case while splitting
it into maximal ASCII-alphanumeric runs. Compare a token's ASCII-lowercase form
against this closed set of formulation words:
`vaccine`, `vaccines`, `human`, `virus`, `viral`, `live`, `inactivated`,
`attenuated`, `recombinant`, `conjugate`, `polysaccharide`, `adsorbed`,
`injectable`, `split`, `valent`, `quadrivalent`, `trivalent`, `dose`, `pf`,
`preservative`, `with`, `without`, and `free`, discarding matches. Retain other
numeric tokens as constraints, but they never qualify a description by
themselves. A retained token is qualifying when either (a) it contains only
ASCII letters and has at least four characters after lowercasing, or (b) its
original spelling matches `[A-Z]{3,}[0-9]*`. Thus `HPV`, `HPV9`, and `PCV` are
qualifying initialisms, while ordinary lowercase three-letter words such as
`abc` are not. A description is usable only when it has at least one
qualifying token; all of its retained tokens, including digits, then form its
ordered signature.

For CVX matching, first require that usability rule. An exact match means the
existing whitespace-collapsed, ASCII-case-insensitive equality between the
complete description and one complete EMA name, active-substance, or
indication scalar. Otherwise compact each retained signature token and the EMA
field to ASCII lowercase alphanumerics. Search one EMA field with a cursor
starting at byte zero: find the first signature token at or after the cursor,
advance the cursor to the byte after that match, and repeat. Every token must
match in the same field and in order; tokens cannot be assembled across EMA
fields. Punctuation and spaces may therefore disappear (`papilloma virus`
matches `papillomavirus`), but order cannot reverse and intervening candidate
text does not matter. This exact-or-ordered-signature rule applies only to
descriptions from qualified CVX rows. It never applies to MyChem aliases or the
requested query.

The deterministic fixtures must prove all three intended bridges:
`Gardasil` admits EMA `Silgard` through the quadrivalent HPV description,
`Prevnar` admits `Prevenar 13` through the pneumococcal/13 description, and
`Fluzone` admits the recorded EMA influenza rows. They must also prove that
descriptions sharing only discarded words such as `vaccine`, `conjugate`,
`trivalent`, `preservative`, or a number do not match. Focused normalization
tests cover literal `HPV`, `HPV9`, and `PCV` signatures, a multi-token compact
match, reversed and cross-field failures, and lowercase `abc` as a non-
qualifying three-letter token.

## EMA matching truth table

Classify a human EMA medicine using the first applicable row below. “Exact”
means equality to the normalized complete scalar field, matching the existing
record-0960 tier definition. A primary phrase uses the existing
alphanumeric-boundary match (including the adapter's existing delimiter
handling); it is not split into individual tokens.

| Precedence | Condition | `match_kind` | `matched_term` | `source` |
| --- | --- | --- | --- | --- |
| 1 | requested query exactly matches EMA `name_of_medicine` | `product_name` | cleaned requested query | `query` |
| 2 | requested query exactly matches EMA `active_substance` | `active_substance` | cleaned requested query | `query` |
| 3 | an allowed MyChem generic/brand term exactly matches EMA name or active substance | `alias` | that cleaned term | its exact source-field name from the table above |
| 4 | a qualified CVX description exactly or distinctively matches EMA name, active substance, or indication | `alias` | the cleaned CVX description | `cvx_short_description` or `cvx_full_vaccine_name` |
| 5 | the requested phrase occurs on boundaries in EMA name, active substance, or therapeutic indication | `broad_text` | cleaned requested query | `query` |
| - | only an untyped alias token, a generic CVX token, or no rule matches | excluded | - | - |

If multiple terms satisfy one row, choose the earliest term in the typed
identity's stable source/row order. A better precedence always replaces a
worse one. Thus the three existing tier labels and record 0960's
`broad_text` label keep their meaning; the additive fields explain why the row
was present.

## Result and pagination contract

Replace the parallel `match_kinds` bookkeeping with per-candidate match
metadata (or an equivalently invariant internal type) so a row cannot become
detached from its explanation. Each EU result in CLI JSON and raw-MCP JSON has
these additive, non-null fields:

```json
{
  "match_kind": "active_substance",
  "matched_term": "eflornithine",
  "source": "query"
}
```

The `source` vocabulary is exactly `query`, `openfda.generic_name`,
`ndc.nonproprietaryname`, `drugbank.name`, `chembl.pref_name`,
`openfda.brand_name`, `cvx_short_description`, or
`cvx_full_vaccine_name`. Existing EMA row fields and the enclosing region
envelope remain unchanged. US and WHO result shapes do not gain
`matched_term` or `source`; their existing `match_kind` remains unchanged.

For EU Markdown, add one `Match` column containing
`<match_kind>: <matched_term> (<source>)`, with the three dynamic values passed
through the existing Markdown-cell escaping. This also appears in raw MCP's
default readable response because raw MCP executes the CLI surface. Do not add
drug to the typed MCP search schema, change its catalog, or add the metadata to
`get drug` Markdown/JSON.

For the complete bounded EMA feed, classify matching candidates, discard
nonmatches, and deduplicate case-insensitively by nonblank EMA product number
before applying user pagination. Record the product's first feed position. If
a later duplicate has a better precedence, replace the entire retained EMA
result row and its matching metadata with that later candidate while retaining
the recorded first position as its stable tie key. On equal precedence keep
the earlier candidate and its metadata. Stable-sort the deduplicated rows by
the existing tiers
`product_name`, `active_substance`, `alias`, `broad_text`; preserve first feed
position inside a tier. Only then apply `offset` and `limit`.

The EU `total` is the complete post-filter, post-dedup count. Existing
pagination computes `has_more` from that total and the returned slice, and the
existing EU-only continuation command advances by the returned count. Empty
and out-of-range pages retain the existing envelope and continuation behavior.

## Acceptance

- A provider-shaped MyChem fixture for eflornithine contains the real
  systematic DrugBank synonym with `acid`, an allowed generic-name field, and
  a verified OpenFDA brand. A provider-shaped EMA fixture contains `Vaniqa`,
  `Prasugrel Viatris`, exact-name/active/brand rows, primary-phrase broad rows,
  duplicates, and later better-tier rows. `Vaniqa` and the legitimate rows
  remain; `Prasugrel Viatris` and rows admitted only by `acid` disappear.
- Unit tests cover every truth-table row, every allowed source value, rejection
  of each excluded MyChem field, independent hit admission, precedence,
  deterministic ties, and classification/dedup/filter-before-pagination. A
  duplicate fixture gives the two rows different visible fields and proves a
  better later row and its metadata replace the earlier row without changing
  its tie position. Pagination tests assert
  exact `total`, `has_more`, and continuation at the first, middle, final,
  empty, and out-of-range pages.
- Recorded CVX/EMA fixtures prove the Gardasil/Silgard,
  Prevnar/Prevenar-13, and Fluzone/influenza positives plus generic-token and
  numeric-only negatives.
- CLI process contracts cover EU and all-region Markdown and JSON. Raw MCP
  contracts cover readable and `json: true` calls and compare their EU match
  facts with the CLI. A schema/catalog contract proves typed MCP still rejects
  `entity: "drug"` and was not widened by this ticket.
- Existing indication-only searches, US and WHO search fixtures, MyChem-empty
  and MyChem-error fallback tests, CVX-error fallback, EMA sync bounds, and
  request-count/receipt tests remain green. No new unbounded provider call,
  retry, or local-data scan is introduced.
- User-facing EMA/CVX drug-search documentation describes primary phrase
  matching, typed aliases, the vaccine bridge, match metadata, and per-region
  pagination without calling a name search an indication search.

## Boundaries and dependencies

This ticket changes EMA name-search identity, classification, presentation,
fixtures, and their docs. It does not change explicit `--indication` meaning,
US/WHO matching, OpenFDA fallback, `get drug`, public `Drug.brand_names`, the
EMA or CVX feeds, sync/network/body/row limits, or MCP tool inventory. It has
no dependency on tickets 1161, 1151, or 1153; those drug interaction,
degradation, and orphan-designation changes are separate surfaces.

## Result

EMA name search now builds a search-only typed identity directly from admitted
MyChem hits, with the accepted closed field/source vocabulary and no shared
all-hits fallback. Untyped DrugBank synonyms and the excluded GtoPDB, UNII,
and ChEBI fields cannot admit a hit or contribute an EMA term. Unresolved,
empty, and failed MyChem lookups keep query-only behavior and use the existing
strict CVX trade-name branch. The CVX bridge implements the bounded retained-
token signature exactly, including qualifying uppercase initialisms, numeric
constraints, compact ordered matching within one EMA scalar, and the specified
formulation-word exclusions.

Every retained EU row carries non-null `match_kind`, `matched_term`, and
`source` values through CLI JSON and raw-MCP JSON. EU Markdown renders the
escaped combined Match column; US and WHO shapes and typed MCP's deliberate
rejection of the drug entity remain unchanged. EMA classification, exclusion,
case-insensitive product-number deduplication, later-better whole-row
replacement, stable tier ordering, totals, and pagination all occur across the
complete bounded local feed before slicing.

Provider-shaped MyChem, EMA, and CVX fixtures cover the eflornithine/acid false
positive, every allowed and excluded source, exact and broad tiers, duplicate
replacement, all pagination positions, and the Gardasil/Silgard,
Prevnar/Prevenar-13, and Fluzone/influenza bridges with adversarial generic,
numeric, reversed, and cross-field negatives. User and source documentation
now describe typed identity, primary-phrase matching, CVX bridging, per-region
pagination, and result provenance. Extracting EMA identity into its own module
brought the legacy EMA source below 1,000 lines and removed its obsolete size
inventory entry; two existing test sidecars were consolidated so the source
package remains at the enforced 1,300-file ceiling.

## Verification

- Test-first proof: the initial focused Rust test did not compile because the
  typed EMA identity constructor did not exist.
- Focused Rust: 11 CVX parsing tests, 12 EMA parsing tests, 2 identity tests,
  8 drug JSON tests, and 19 drug Markdown tests passed; the complete
  `entities::drug` library lane passed 80 tests. After the package-neutral
  file consolidation, all 17 EMA tests and all 15 drug-search tests passed.
- Remediation Rust: all 17 drug-search tests passed, including one independent
  admission case for each allowed MyChem field and a public EU orchestration
  fixture covering malformed, empty, irrelevant, and excluded-only MyChem
  responses. Each orchestration case asserted a one-request delta and a
  positive CVX result. All 13 EMA parsing tests and the complete 82-test drug
  library lane passed. The get-path regression pins legacy `normalize_term`
  case/whitespace/period handling and alias deduplication separately from the
  search-only cleaner. Private EMA, CVX, and HTTP-cache fixture roots keep the
  orchestration test unpaced and isolated; its four searches finished in 0.15
  seconds after compilation.
- Focused Python/docs: 38 tests passed across the source-page, public-skill,
  and upstream-planning documentation contracts. The source capture receipt
  audit passed.
- Executable contract: the focused provider/DDInter-backed
  `spec/entity/drug.md` run passed all 13 examples, including CLI Markdown and
  JSON, raw MCP readable and JSON modes, typed-MCP rejection, false-positive
  exclusion, and pagination surfaces. Manual fixture-backed CVX checks returned
  Silgard, Prevenar 13, and the recorded influenza products with the expected
  CVX sources.
- The remediation contract adds exact public first-, middle-, and final-page
  assertions for `total`, `has_more`, and continuation offsets. Its focused
  provider/DDInter-backed run passed all 13 examples after rebuilding the
  remediated CLI and MCP example.
- Quality/package: the final `make lint` invocation passed its credential,
  documentation, fixture, Python, shell, formatting, Clippy-with-warnings-
  denied, license, and advisory checks, then failed only because the quality
  ratchet correctly saw an intentional deleted test sidecar as still tracked
  before staging. After staging that deletion, the complete standalone quality
  ratchet and `git diff --cached --check` passed. An earlier complete lint run
  after the EMA extraction had passed the same ratchet. `cargo build
  --no-default-features --bin biomcp` passed. `cargo package --list
  --allow-dirty --locked --offline --no-verify` reports exactly 1,300 files.

The first package-list attempt exposed two new packaged sidecars and 1,302
paths; consolidating the new identity tests and an existing EMA construction
test sidecar restored the exact ceiling. A direct verified `cargo package`
cannot run because the repository intentionally uses the existing Git-only
BioData dependency without a registry version; the repository-standard
  offline package-list check passed. Full `make test`, full `make spec`, the
  full-feature lane, and the release gate were not run, and no merge was
  performed.
- Remediation final checks: `make lint` passed completely, `git diff --check`
  passed, and the offline package-list count remained exactly 1,300 files.
- Final proof remediation: the all-region JSON unit now compares the complete
  EU result, including `match_kind`, `matched_term`, and `source`, and compares
  the complete unchanged US and WHO results. A deterministic Herceptin fixture
  lets the executable `--json search drug trastuzumab --region all` contract
  assert the same EU provenance while also asserting nonempty US and WHO rows
  retain `match_kind` without gaining `matched_term` or `source`. The focused
  Rust unit passed, the provider/DDInter-backed drug spec again passed all 13
  examples, `cargo fmt --check`, `git diff --check`, and the complete `make
  lint` gate passed.

## Review

- Design review: accepted in the materially detailed ticket before
  implementation.
- First implementation review: rejected. The initial record overstated
  independent coverage of every allowed admission field and the four full
  unresolved-MyChem orchestration paths; those cases existed only at lower
  layers or not at all. It also failed to pin the legacy get-path
  normalization boundary and exact middle-page continuation behavior.
- Remediation: added the missing independent and fixture-driven coverage,
  separated the search-only identity constructor from the unchanged legacy
  get constructor, and strengthened the public pagination contract.
- Second independent implementation rereview accepted the implementation
  behavior and found one remaining proof gap: no executable all-region JSON
  assertion jointly pinned EU provenance and the legacy US/WHO row shapes.
  The final proof remediation adds that contract and its exact unit analogue.
  Fresh independent implementation rereview is pending; no merge was
  performed.
