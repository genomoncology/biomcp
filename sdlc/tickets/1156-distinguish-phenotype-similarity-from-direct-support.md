---
flow: build
priority: 9
deps: [1157]
---

# Distinguish phenotype similarity from direct support

## Goal

Phenotype search must not present ontology similarity as proof that a disease
has the requested phenotype. On 2026-09-04,
`biomcp --no-cache search phenotype HP:0000256 --limit 5` ranked isolated
microcephaly first for a macrocephaly query. Monarch's direct disease
associations contained microcephaly and did not contain macrocephaly, but
BioMCP still recommended the first row as the next disease to open. The
reproduction and provider evidence are recorded in
`sdlc/issues/2026-09-04-phenotype-similarity-ranks-an-opposite-phenotype-without-warning.md`
at commit `84f2343f`.

## Settled contract

### Resolved query identity

Every successful phenotype search exposes a top-level `resolved_query` array.
Each entry has exactly this ordered shape:

```json
{"raw":"macrocephaly","id":"HP:0000256","label":"Macrocephaly"}
```

The array order is the order of the unique normalized HPO IDs sent to the
Monarch similarity request. For direct-ID input, `raw` is the first submitted
token that produced the ID; normalization and duplicate removal retain the
first occurrence. For free text, `raw` is the comma-delimited symptom phrase
that produced the HPO search row, and HPO result order is retained. An ID
resolved from more than one phrase retains the first phrase and first row.
Reject more than ten comma-delimited phrases before provider contact. Resolve
accepted phrases with at most four HPO requests in flight, then flatten valid
HPO rows in input-phrase and provider-row order and normalize and deduplicate
them by first occurrence. Do not stop processing just because ten unique IDs
have already been seen. If the complete flattened result contains more than
ten unique HPO IDs, reject the whole query with an invalid-argument error that
says the search resolved more than ten unique terms and asks the user to refine
the symptom phrases. Make no Monarch request. There is no per-phrase quota and
no truncation: silently dropping the eleventh ID could remove the only term
contributed by a later valid phrase and would make the direct-support claim
describe only part of the user's query. Every successful query therefore has
at most ten unique resolved IDs and retains all of them in the deterministic
order above.

Every accepted free-text phrase must resolve at least one HPO row. If one or
more phrases return a successful zero-row response, fail the whole search with
the existing typed no-HPO-match invalid-argument family, name the original
unresolved phrases in input order, and make no Monarch request. Do not silently
drop an unresolved phrase merely because another phrase resolved. Resolution
may issue the bounded HPO searches concurrently, but it collects their
outcomes before deciding whether the input is valid; provider failure remains
the typed HPO provider error described below rather than being reported as a
zero-row match. Thus a successful `resolved_query` accounts for every submitted
non-empty comma-delimited phrase, although multiple phrases can still resolve
to the same first-occurrence-deduplicated HPO ID.

For each free-text HPO search, the response envelope must contain a `terms`
field whose value is an array. The decoder preserves field presence rather
than defaulting a missing field to an empty vector. A present empty array is a
valid zero-row response and follows the unresolved-user-input rule above;
missing `terms`, `terms: null`, or any other non-array value is a typed HPO
provider/decode failure, takes precedence over any simultaneous zero-row
phrase, and makes no Monarch request. It must never be reported as an
unresolved user phrase. After all bounded HPO operations settle, apply
validation in this deterministic order: provider/transport/decode failures,
then zero-row phrases in input order, then the aggregate ten-unique-ID bound.

`label` comes only from the HPO API: the matching HPO search row for free text
and `GET /terms/{id}` for direct IDs. A blank label, a term response whose
normalized ID does not equal the requested ID, a 404 for a submitted direct
ID, or a transport, status, content-type, or decode failure is not replaced by
a disease label, the raw query, or the CURIE. It fails the search with the
existing typed HPO error before either Monarch request. Successful output
therefore always has a non-empty label for every resolved term.

### Per-term direct-support states

Each result retains the Monarch semantic-similarity `score` and adds a
`direct_support` array in the same order as `resolved_query`. Each entry is
`{"hpo_id":"HP:0000256","status":"..."}` and `status` is one of the
following four strings only:

- `supported`: the association response contains at least one row whose
  subject is this result's exact MONDO ID, object is this exact resolved HPO
  ID, category is
  `biolink:DiseaseToPhenotypicFeatureAssociation`, predicate is
  `biolink:has_phenotype`, and `negated` is not `true`. A related, ancestor,
  descendant, similarly named, or differently predicated HPO row is not
  support.
- `not_supported`: the direct-association lookup is complete and contains no
  positive row meeting the rule above for this exact disease/HPO pair. A row
  with `negated: true` is not positive support; on an otherwise complete
  lookup the pair is `not_supported`.
- `indeterminate`: no positive row was found for the pair, but the lookup
  cannot prove absence because `total` exceeds the returned item count or the
  response is internally inconsistent or contains a row that violates the
  requested subject/object/category/predicate filter. Valid positive rows may
  still be `supported` in the same truncated response, but every unmatched
  pair is `indeterminate`.
- `unavailable`: the association request was not completed because of its
  deadline, transport, non-success status, content-type, body-limit, or JSON
  decode failure. The similarity rows remain a successful degraded response,
  every pair is `unavailable`, and the failure is summarized without raw
  provider bodies or local details.

The association wire decoder tracks the presence of `total` and `items`
separately; it must not use Serde defaults or an empty-vector/zero fallback that
erases absence. Only a successful, filter-consistent response in which `total`
is present as a non-negative integer, `items` is present as an array,
`total == items.len()`, and `total <= 500` is complete enough to produce
`not_supported`. An explicitly present `{"total":0,"items":[]}` is complete.
If either field is absent, the lookup is incomplete: independently valid
positive rows from a present `items` array may still produce `supported`, but
all unmatched pairs are `indeterminate`; absent `items` means there are no
rows from which to establish `supported`. A present field with the wrong JSON
type is a decode failure and therefore `unavailable`. Neither missing fields,
truncation, internal inconsistency, nor provider failure may be converted into
a negative association.

### Exact bounded association lookup

Ticket 1157 remains the owner of candidate retrieval and paging. First fetch
and normalize its one fixed Monarch similarity window, then apply the local
`offset` and `limit`, and only then enrich the returned slice. For a non-empty
slice, make exactly one additional Monarch request with this parameter set:

```text
GET /v3/api/association
subject=<one repeated parameter for each sliced MONDO ID, in slice order>
object=<one repeated parameter for each resolved HPO ID, in resolved_query order>
category=biolink:DiseaseToPhenotypicFeatureAssociation
predicate=biolink:has_phenotype
object_category=biolink:PhenotypicFeature
direct=true
limit=500
offset=0
```

Do not send unsliced candidates, duplicate subject/object values, a broader
`entity` query, omit `direct=true`, or fetch a second association page. An
empty result slice makes zero association requests. The maximum cross-product
is 50 sliced diseases by 10 resolved HPO IDs; multiple source rows can still
make `total` exceed 500, which is why absence then remains `indeterminate`.

The direct-support phase has one in-flight logical association operation and
one non-configurable eight-second wall-clock deadline covering middleware
retries and body reading. Deadline expiry cancels the operation and produces
`unavailable`; it does not delay the response until the shared client's longer
timeout. HPO phrase searches or direct-ID label lookups use at most ten
logical source operations, at most four in flight, and one shared eight-second
wall-clock resolution deadline; expiry or any failed/malformed label fails the
command as described above. Including ticket 1157's one logical similarity
operation and this ticket's one logical batched association operation, a
successful invocation starts at most 12 logical source operations and has at
most four in flight.

All 12 operations retain the shared client's existing policy of one initial
HTTP attempt plus at most three retries for retryable failures. Therefore the
hard upper bound is 48 physical HTTP attempts per invocation (40 HPO, four
similarity, and four association), with phase and whole-command deadlines able
to cancel attempts or retry sleeps earlier. A cache hit still counts as one
logical operation but can require no physical provider attempt. The ticket
does not add another retry loop around the shared client.

A non-configurable 30-second wall-clock deadline covers all phenotype provider
work; expiry during HPO resolution or similarity retrieval is a typed provider
error, while expiry during optional direct-support enrichment returns the
similarity page with `unavailable`. These bounds are constants and are covered
by tests rather than new public configuration. The eight-second enrichment
budget follows the existing disease optional-enrichment policy, and the outer
30-second ceiling does not exceed the shared HTTP client's request timeout.

### Rendering and follow-up commands

CLI Markdown prints the ordered resolved terms before the table, calls the
ranking semantic similarity, and shows every row's per-HPO support states.
`not_supported` is phrased as “no direct Monarch association in the complete
lookup,” not as proof that the disease lacks the phenotype. `indeterminate`
and `unavailable` render distinct warnings. JSON uses the exact fields and
closed strings above; it must not infer a row-level “match” boolean that hides
mixed multi-term states.

Continuation command construction, provider order, and the 1157 pagination
and provider-window metadata stay byte-for-byte equivalent apart from the new
query/support fields around them. The disease follow-up is deterministic: scan
the sliced rows in provider order and emit
`biomcp get disease <MONDO_ID> phenotypes` only for the first row whose every
`direct_support` entry is `supported`. Use the stable disease ID, not the
provider label. If no returned row supports every resolved term, suppress the
disease follow-up and say why in Markdown; never fall back to the first
similarity row. The pagination continuation remains first when present, and
the existing `biomcp list phenotype` helper remains after any disease
follow-up. Markdown and JSON derive these commands from the same selector.

The scope is CLI Markdown, CLI JSON, and raw MCP `biomcp` calls in both default
and `json:true` modes. Raw MCP inherits the same renderer/payload and error or
degradation behavior. Phenotype remains absent from the typed MCP `search`
schema.

## Required proof

Extend the supervised, fail-closed phenotype fixture and
`spec/entity/phenotype.md`; do not use a live request. The fixture accepts only
the exact HPO, 1157 similarity, and association routes named by the cases and
returns 404 for broadened filters, omitted filters, alternate limits/offsets,
extra calls, and association subjects outside the locally sliced page. Use
per-case request-log deltas so parallel spec pages cannot satisfy counts with
one another's traffic.

The fixed adversarial matrix covers all of the following:

- The macrocephaly similarity response ranks isolated microcephaly first. Its
  complete direct lookup has no `HP:0000256` association, so the row is
  `not_supported`; a later disease has an exact positive macrocephaly row and
  is `supported`. Markdown does not call the first row a match, and the next
  disease command uses the later supported MONDO ID.
- A complete zero-item association response produces `not_supported`, while a
  `negated: true` exact row does not become `supported`.
- Successful association JSON with missing `total`, missing `items`, and both
  fields missing proves the decoder retains presence. None of those fixtures
  can produce `not_supported`; present valid rows with missing `total` can
  still produce `supported`, while every unmatched pair is `indeterminate`.
  The explicit `{"total":0,"items":[]}` control still produces
  `not_supported`.
- A response with `total > items.len()` makes unmatched pairs
  `indeterminate`, retains any independently valid `supported` pair, and emits
  no unsafe disease command unless one row supports every term.
- Association transport, HTTP status, content-type, body-limit, decode, and
  eight-second deadline failures are unit-tested as `unavailable` degradation,
  never `not_supported` or a failed similarity search. At least one fixed
  provider-failure route is exercised through CLI and raw MCP.
- A multi-term query proves exact pairwise behavior: one disease supports only
  the first term, another supports both, array order matches `resolved_query`,
  and only the latter is eligible for the follow-up command.
- Direct-ID input proves ordered first-occurrence deduplication, exact
  `/terms/{id}` label requests, non-empty HPO-sourced labels, the unchanged
  one `limit=50` similarity request, and exactly one post-slice association
  request. Label 404/mismatch/blank/failure proves that no Monarch request is
  made.
- Free-text `macrocephaly` proves its ordered `{raw,id,label}` resolution from
  HPO search and then the identical similarity/direct-support contract, with
  no redundant term-label request.
- Mixed free text such as `macrocephaly, phrase-with-no-hpo-row` proves that a
  successful zero-row response for the second phrase fails the whole search,
  names that original phrase, and makes no Monarch similarity or association
  request; the resolved first phrase is not silently accepted on its own.
- Two valid phrases whose provider-ordered rows collectively contain eleven
  unique HPO IDs prove that resolution flattens by phrase then row, detects the
  eleventh first occurrence, rejects instead of truncating or allocating a
  per-phrase quota, and makes no Monarch request. A companion case with ten
  unique IDs, including a cross-phrase duplicate, succeeds and pins
  first-occurrence order and the duplicate's first phrase in `raw`.
- Free-text HPO envelopes with missing `terms`, `terms: null`, and a non-array
  `terms` value each produce a typed HPO provider/decode failure and no Monarch
  request. The explicit `{"terms":[]}` control produces the distinct typed
  no-HPO-match invalid argument, proving malformed provider data cannot be
  mistaken for unresolved user input. A mixed concurrent case with one empty
  array and one malformed envelope proves the provider failure wins.
- Limits one, two, three, and five plus supported offsets retain ticket 1157's
  fixed order, first-occurrence deduplication, tied-row order,
  `provider_window_limit`, `provider_raw_row_count`,
  `provider_window_exhausted`, `has_more`, and continuation window. Each run
  has one `limit=50` similarity request; its association request contains only
  that run's sliced, normalized disease IDs. An empty slice has none.
- CLI Markdown, CLI JSON, raw MCP default, and raw MCP `json:true` agree on
  resolved terms, all four support states, safe command selection/suppression,
  warnings, and the unchanged 1157 window metadata. A source assertion keeps
  phenotype out of the typed MCP schema.
- Request construction and parsing unit tests pin repeated-parameter order,
  all filters, `limit=500`, `offset=0`, completeness checks, response-row
  validation, fail-closed `total`/`items` and HPO `terms` presence tracking,
  the ten-phrase pre-contact rejection, deterministic aggregate over-ten
  rejection without truncation, the 12-logical-operation/48-physical-attempt
  upper bounds, concurrency constants, absence of a nested retry loop, and
  both phase and whole-command deadline behavior.

Update `docs/user-guide/phenotype.md` and
`docs/sources/monarch-initiative.md` to explain that results are semantic
similarity candidates, define the four direct-support states, show resolved
HPO IDs and labels, explain that only complete lookups yield
`not_supported`, and describe when disease follow-ups are suppressed. Keep the
data-source/reference wording consistent if it describes phenotype search.

## Boundaries

This ticket qualifies the locally sliced semantic-similarity results with
public, exact direct-association evidence. It does not diagnose a patient,
invent antonym or ontology-proximity rules, change or re-sort Monarch's
similarity score, alter ticket 1157's normalized window or paging semantics,
fetch support for rows outside the requested page, follow association
pagination beyond the fixed bound, rank a complete patient phenotype profile,
or add a typed MCP phenotype-search surface.
