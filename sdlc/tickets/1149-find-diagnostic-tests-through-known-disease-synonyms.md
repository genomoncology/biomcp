---
flow: build
priority: 6
deps: []
---

# Find diagnostic tests through known disease synonyms

## Outcome

`biomcp search diagnostic --disease "Bachmann-Bupp syndrome" --source gtr`
returns `GTR000596648.2`, whose GTR condition uses a longer exact synonym.
Each returned row says whether it matched the requested term, the canonical
disease name, or a known synonym. The search never broadens through parents,
descendants, fuzzy matches, substring candidate selection, or `discover`.

## Current facts

- Diagnostic disease matching currently checks one literal phrase against
  each local condition in `src/entities/diagnostic/search.rs`.
- `src/entities/disease/resolution.rs` already owns MyDisease identity
  resolution, but its public disease-card path may use fuzzy ranking and a
  discover fallback; diagnostic expansion must not call that broad path.
- GTR conditions are ontology disease labels. WHO IVD's target/marker field is
  not, so WHO must continue matching only the caller's literal phrase.
- Diagnostic search is not currently a typed MCP search branch. Adding one is
  unrelated to this bug.

## Design

Add a small exact resolver in `entities::disease::resolution` and reuse the
existing MyDisease client. It accepts the validated disease text and returns
the requested term plus, when one exact identity is found, its canonical ID,
canonical name, and at most twenty valid exact synonyms.

For free text, retain only provider hits whose primary MONDO/DO name or declared
synonym equals the query after trimming, ASCII case-folding, and collapsing
Unicode whitespace. If exactly one canonical identity remains, fetch its
detail once; zero or multiple identities use the literal term only. A direct
MONDO/DOID query may fetch that detail directly. Do not add MESH/OMIM/ICD
crosswalk behavior in this ticket.

The resolver may make at most two logical requests and uses the client's
existing cache, transport, response-size, and timeout policies. Validate the
disease filter before local or remote work: after trimming it must be at most
512 UTF-8 bytes, contain no control characters, and retain the existing
three-alphanumeric minimum. Invalid provider identity text is ignored. A
resolver transport, HTTP, decoding, or selected-detail consistency failure is
returned as the existing safe MyDisease source error; it must not become a
confirmed empty diagnostic result.

GTR matches the requested term, canonical name, then synonyms in provider
order using the existing Unicode-safe alphanumeric phrase boundary. Record the
first match on the result as an optional structured `disease_match` with
`kind` (`requested`, `canonical`, or `synonym`), the preserved term, and the
resolved canonical ID when available. WHO uses only the requested term and
never invokes MyDisease. No disease filter means no resolver call and no
`disease_match` field.

Rank disease-filtered GTR rows by match kind and synonym order, then preserve
the current name/accession ordering. Apply every gene, type, manufacturer, and
source filter conjunctively; expansion cannot revive a row rejected by another
filter. Match and rank before the existing offset/limit slice. Preserve
current cross-source behavior, JSON pagination shape, next commands, and all
non-disease-filter ordering.

JSON, Markdown, and raw MCP use the existing diagnostic serializers. Markdown
adds a compact `Disease match` column only for disease-filtered results. Escape
provider text through the established table renderer; do not introduce a new
renderer or command builder.

## Acceptance

- Pure tests cover exact-name/synonym selection, ambiguity/absence, bounded
  synonym normalization, no fuzzy/parent expansion, and invalid inputs before
  work.
- Fixture-backed CLI JSON and Markdown prove the Bachmann-Bupp result, match
  provenance, literal long-name compatibility, conjunctive filters, stable
  ordering, and offset/limit behavior.
- WHO-only and no-disease searches make zero MyDisease requests. Resolver
  failure is a safe error rather than a false zero.
- Raw MCP JSON and Markdown remain byte-equivalent to their CLI bodies after
  the existing footer rules. Existing diagnostic, disease, and package
  contracts remain green.
- Run focused Rust and executable diagnostic specs, then `make lint`, `make
  test`, `make spec`, the exact 1,300-path package inventory, and `git diff
  --check`.

## Boundaries

No ontology traversal, fuzzy matching, xref crosswalk expansion, typed MCP
branch, new provider, local-data ingestion change, partial-source recovery
redesign, dependency, fixture family, or new packaged file. Keep changes in
the existing disease resolver, diagnostic owners, fixture, docs, and spec.
Do not raise a source-size or CLI line-count allowance.

## Complexity

- Contract: 1; state/timing: 1; reach: 1; proof: 1; cost of error: 1.
- Total: 5; level 3.
- Selected model: GPT-5.6 SOL Medium. The implementation crosses a remote
  resolver and local search/pagination boundary, so this is one of the hard
  tickets for which the SOL coding route is appropriate.

## Review

The earlier accepted design was superseded before implementation because it
combined this synonym bug with crosswalks, partial-source recovery, typed MCP,
and byte-level adversarial matrices. Fresh review pending on this focused
contract.
