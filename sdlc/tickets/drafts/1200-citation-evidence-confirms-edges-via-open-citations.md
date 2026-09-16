---
flow: build
priority: 3
deps: []
---

# 1200: Citation evidence confirms edges through an open citation index

## Goal

`biomcp article citation-evidence <citing-id> <cited-id>` gains a sixth outcome
state, `reference_confirmed_without_passage`: when the passage is unavailable
but OpenCitations confirms the directed edge, the command returns the
confirmed citation record instead of a bare `fulltext_unavailable`, so a
caller can distinguish "the passage is not openly available" from "this
citation may not exist."

## Current Facts

The five-state outcome is frozen by ticket 1145. The `fulltext_unavailable`
dead end carries no edge information: when the Semantic Scholar edge has no
contexts and the Europe PMC JATS attempt fails, the caller learns only that
full text was unavailable.

Semantic Scholar context coverage is partial (a 2026-09-04 research run found
useful context on 23 of 61 incoming edges). A live smoke test on 2026-09-16
hit the dead end on a well-known pair (`22663011 -> 10.1038/nature10725`,
NEJM citing a Nature paper): the Semantic Scholar references endpoint returned
`data: null` for the citing paper while the full text was paywalled.

OpenCitations (live, 2026-09-16): `GET /index/api/v2/references/doi:<doi>` on
`https://api.opencitations.net` returns a JSON array of citation rows, each
with string `oci` (e.g. `0680682894-06220343544`), string `citing` and string
`cited` (whitespace-separated identifier lists such as
`omid:br/061502131318 doi:10.1056/nejmoa1203421 openalex:W2136474966
pmid:22663011`), string `creation` (day or month precision, e.g. `2012-07-12`
or `2011-03`), `timespan`, `journal_sc`, `author_sc`. Unknown or malformed
DOIs return HTTP 200 with `[]` (not 404). A 294-row response measured 108 KB;
latency measured under one second. No authentication is required. The
provider does not carry passages; nothing in this ticket tries to recover text
from it.

Ticket 1186's `sanitize_provider_inline` and 1145's provider-sanitization
discipline apply to every provider string that reaches output.

## Design

### Source module

New module `src/sources/opencitations.rs`, owned by the sources layer,
following the existing client shape (shared client, `apply_cache_mode`,
`env_base`, `read_limited_source_body` with `DEFAULT_MAX_BODY_BYTES`):

- Base: `OPENCITATIONS_BASE = "https://api.opencitations.net/index/v2"`,
  overridable by `BIOMCP_OPENCITATIONS_BASE` for fixtures.
- Request: `GET /references/doi:<normalized-citing-doi>` — one request per
  command at most, placed after the JATS attempt and under the remaining
  absolute command deadline. No pagination: the provider returns the full
  known reference list in one response.
- Result type: `Vec<OpenCitationsEdge>` with `oci`, `citing`, `cited`
  (strings) and `creation` (`Option<String>`). Validation: the response must
  be a JSON array; every element must be an object with string `oci`,
  `citing`, and `cited`. Any malformed element, a non-200 status, a transport
  failure, an oversize body, or a JSON decode failure makes the attempt
  `unavailable`. These are bounded phase failures, never command errors.
- DOI normalization, the same rules as the JATS reference extractor
  (`transform::article::jats::refs`): Unicode-trim; strip once or repeatedly
  the case-insensitive prefixes `doi:`, `https://doi.org/`,
  `http://doi.org/`, `https://dx.doi.org/`, `http://dx.doi.org/`;
  ASCII-lowercase; the remainder must begin `10.` and contain `/`. A citing
  or cited paper without a normalizable DOI makes the phase
  `not_requested`.

### Match rule

Split the row's `cited` string on ASCII whitespace. A token matches when it
equals `doi:<normalized-cited-doi>` ASCII-case-insensitively. First matching
row in provider order wins. No other token form (`omid:`, `openalex:`,
`pmid:`) is consulted.

### Outcome composition

- The phase runs exactly when the assembled outcome would otherwise be
  `fulltext_unavailable` (the contextless path and the forced `--fulltext`
  path, including `ParsedUnusable`). It does not run for
  `context_from_provider`, `context_from_fulltext`, `reference_unresolved`, or
  `citation_marker_unlinked`.
- Confirmed: status `reference_confirmed_without_passage`, source
  `opencitations`, message exactly
  `An open citation index confirmed this directed edge, but no open passage is available.`
  Passages stay empty; `fulltext_locator` stays null; `provider_contexts`
  keep whatever the caller path already retained.
- Not confirmed (no matching row), or the attempt was `unavailable` or
  `not_requested`: the outcome stays exactly today's `fulltext_unavailable`
  with its exact message, source null, and `confirmation` null.
- If the remaining deadline leaves no room to start the request, the phase
  reports `unavailable` and the outcome stays `fulltext_unavailable`.
- The Semantic Scholar directed `NotFound` error path is unchanged: an
  exhausted reference traversal without the edge remains the existing command
  error. (`data: null` from Semantic Scholar remains the existing decode
  error per 1145.)

### Frozen JSON additions

Every state gains exactly one member, always present and nullable:

```json
"confirmation": {
  "source": "opencitations",
  "oci": "061502131318-062102119315",
  "citing": "doi:10.1056/nejmoa1203421",
  "cited": "doi:10.1038/nature10725",
  "creation": "2012-07-12",
  "evidence_url": "https://api.opencitations.net/index/v2/references/doi:10.1056/nejmoa1203421"
}
```

- `citing` and `cited` are locally constructed from the command's own normalized
  DOIs, never echo provider text.
- `oci` is the matching row's `oci`, accepted only when it matches
  `^[0-9]{1,32}-[0-9]{1,32}$`; a matching row with a malformed `oci` makes the
  attempt `unavailable` (fail closed, no confirmation).
- `creation` copies the row only when it matches `^\d{4}-\d{2}(-\d{2})?$`;
  otherwise null.
- `evidence_url` is always the production canonical URL constructed from the
  normalized citing DOI, never the fixture base.

`_meta.source_status` gains a third row in **every** state, appended last:
`{"source":"opencitations","status": <not_requested|available|unavailable>}`.
`available` means a 200 body was successfully parsed (regardless of match);
`unavailable` covers transport, status, oversize, decode, or malformed-row
failure and the no-room deadline case; `not_requested` covers every state
where the phase does not run and the missing-DOI case.
`_meta.evidence_urls` appends
`{"source":"opencitations","url": <the canonical URL>}` when a 200 body was
successfully parsed (mirroring the JATS append rule).

### Markdown

The existing renderer layout renders the new state with its new `Status:`
line; `Full text:` is omitted because the locator is null; no new Markdown
sections and the `confirmation` member is JSON-only, consistent with `_meta`
being JSON-only today. The five existing Markdown goldens are unchanged.

## Fixtures

Focused tests:

1. Wire tests for the client: request path construction from normalized DOIs,
   DOI normalization table (every prefix, repeated prefix, case, leading
   zeros untouched, non-DOI rejection), array/row validation, malformed-row
   handling, non-200, oversize, and deadline behavior through the loopback
   fixture.
2. Match tests: first-match-wins with duplicate rows, case-insensitive DOI
   token match, non-matching rows, empty array, and `omid:`/`openalex:`/
   `pmid:`-only rows that never match.
3. Composition tests: confirmed upgrade on the contextless path; confirmed
   upgrade on the forced path with retained provider contexts; no-match and
   failed attempts keep `fulltext_unavailable`; the phase never runs for the
   four non-`fulltext_unavailable` outcomes; `not_requested` when either side
   lacks a DOI; `NotFound` remains a command error.
4. Hostile fixtures: provider `oci`/`creation` values with quotes, pipes,
   backticks, `$`, `;`, `&`, and non-ASCII scalars fail OCI/creation
   validation or are never echoed (every emitted string is validated or
   locally constructed).

Executable spec (`spec/entity/article.md`, extending the existing capture
fixture family with an OpenCitations route keyed by
`/index/api/v2/references/doi:<doi>` and exporting
`BIOMCP_OPENCITATIONS_BASE`):

- JSON block for the confirmed state: `status`, `message`, `source`,
  `confirmation` object, three-row `statuses`, and `urls` array.
- Markdown block for the confirmed state, byte-exact.
- The existing block that projects the full `._meta.source_status` array
  (the `context_from_fulltext` block) updates to include the third row; every
  other existing projection is key-scoped and unchanged.
- A request-log block proving: no OpenCitations request in a
  `context_from_provider` state, exactly one in the confirmed state, and none
  in a `reference_unresolved` state.
- Raw MCP parity: if the five states already have a raw-tool case in
  `spec/surface/mcp.md`, mirror it for the confirmed state; the implementer
  follows whatever the existing five-state coverage does and reports it.

Docs: `docs/user-guide/article.md` states the five values; it lists six after
this ticket with the new state's one-line meaning.

## Acceptance

The sixth state renders and serializes exactly as specified; wire,
composition, and hostile tests pass; the spec blocks pass; `make lint`,
`make test`, and `make spec` pass on the gate host at the pushed SHA. The
package path count gains exactly the one new source module (authorized
baseline pattern with measured delta and removal condition if any source-size
baseline moves). Ticket 1145's request budgets change only by the one
additional OpenCitations request on the `fulltext_unavailable` path; request
logs prove the exact counts. Typed MCP schemas and the seven-tool catalog are
byte-for-byte unchanged.

## Dependencies

Ticket 1145's five-state contract is the base and must remain intact.

## Boundaries

No passages from OpenCitations (the provider has none). No new typed MCP tool
and no catalog change. No change to the Semantic Scholar traversal, the JATS
extractor, the deadline architecture, or the other five states' messages,
sources, or nullability. No OpenAlex integration in this ticket (OpenCitations
alone; an OpenAlex fallback would be a follow-up). No `data: null` recovery:
Semantic Scholar's decode-error path stays a command error. No summarization
or interpretation of any kind. No change to `article citations` /
`article references` surfaces.

## Complexity

- Contract score: 2 (a new compatibility decision extending a frozen enum and
  JSON surface)
- State and timing score: 1 (one ordered provider phase under the existing
  absolute deadline)
- Reach score: 1 (sources, entity outcome, renderer, spec, docs)
- Proof score: 2 (byte-exact goldens for a new state, hostile provider rows,
  request-log budgets)
- Cost of error score: 1 (a false confirmation would be a false evidence
  claim; user-visible, recoverable, no data loss)
- Total: 7
- Minimum level floor: none
- Final level: 3
- Reasons: frozen-contract extension with a hostile-input proof tier
- Selected model: gpt-5.6-sol, medium reasoning (level 3 implementer)

## Review

- Design review: pending
- Code review: pending
