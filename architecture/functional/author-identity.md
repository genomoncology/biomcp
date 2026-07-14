# Author Identity and Publication Surface

Status: target architecture. BioMCP does not yet ship an author entity. The
article fidelity prerequisites are shipped; the build sequence in ticket 516
adds this surface incrementally.

## Problem and evidence

BioMCP currently has publication-scoped names, affiliations, ORCIDs, and MeSH,
but no researcher entity. `search article --author` performs provider author-
field searches and then drops which author matched. A normalized name is not a
person identifier, PubMed has no author identifier, Semantic Scholar can split
or merge people, and ORCID is strong but incomplete and assertion-level
provenance matters. A flat publication list would also hide independent source
pagination and failures. Coauthor and topic counts would then amplify uncertain
membership.

The target therefore adds `author` beside `article`; it does not widen
`ArticleSearchFilters`, mint a BioMCP global person ID, or turn a citation's
affiliation into a current profile.

### Traced cases (2026-07-14)

These live observations motivate deterministic fixtures; they are examples, not
facts that routine tests should fetch from public APIs.

| Case | PubMed evidence | Semantic Scholar evidence | ORCID evidence | Required interpretation |
|---|---|---|---|---|
| ORCID-backed identity: Atul J. Butte | Recent citations include ORCID `0000-0002-7433-2740` and UCSF affiliation; some citations omit the ORCID | Search returns several records; `semanticscholar:1716151` has 548 papers but its `externalIds` contains DBLP, not ORCID | Public record `orcid:0000-0002-7433-2740` names Atul Janardhan Butte and carries source-labelled UCSF employment | ORCID and a citation can link exactly. S2 cannot be linked merely from the similar name; shared publication evidence is required and S2 split records must remain visible. |
| No ORCID on citation: Louis S. Williams, Cleveland Clinic | PMIDs `42146891`, `42056076`, `40717004`, `40221882`, and `36966011` match `Williams LS[au] AND Cleveland Clinic[ad]`; the matched citation author has no ORCID | `semanticscholar:2269573451` owns PMIDs `42146891` and `42056076`; older myeloma papers appear under `semanticscholar:1994488914`, showing a likely split | ORCID expanded search returns seven Louis Williams records but none supplies evidence tying it to these Cleveland Clinic citations | The S2 records are exact provider identities; their shared name/topic is not enough to link them. PubMed membership is `name_affiliation`, not ORCID-backed. |
| Same-name people | Cleveland Clinic hematology citations identify a Louis S. Williams only by name plus historical publication affiliation | S2 search returns multiple Louis/L. Williams records | `orcid:0000-0002-0220-9057` claims ophthalmology/glaucoma works and is distinct from the Cleveland Clinic hematology corpus; other Louis Williams ORCIDs also exist | Search returns separate candidates or an ambiguous/name-only group. It must not select an ORCID for the clinician. |
| Provider disagreement | PubMed places the Cleveland Clinic author on both newer and older papers | S2 divides the apparent corpus across at least `2269573451` and `1994488914` | No corroborating ORCID is present | Expose two S2 records and shared-PMID evidence; do not auto-merge. |

Live probes used only public records and intentionally inspected no email,
private profile, homepage, or inferred demographic field.

## Public grammar

```text
biomcp search author -q <name> [--affiliation <text>] [--source <all|semanticscholar|pubmed>] [--limit N] [--offset N]
biomcp get author <semanticscholar:ID|orcid:ORCID>
biomcp author publications <provider-qualified-id> [--source <auto|semanticscholar|pubmed|orcid|all>] [--limit N] [--offset N]
biomcp author coauthors <provider-qualified-id> [--source <auto|semanticscholar>] [--max-publications N] [--limit N] [--offset N]
biomcp author topics <provider-qualified-id> [--source pubmed] [--max-publications N] [--limit N] [--offset N]
```

`search/get author` join the existing typed entity grammar. The three read-only
helpers form `Commands::Author` and must be admitted explicitly by the MCP
subcommand allowlist. There is no unqualified or BioMCP-minted author ID.
`pubmed:` is not accepted by `get author` because PubMed has no author entity ID.
A PubMed name result instead supplies refinement commands and publication-level
evidence. `--affiliation` is a capability constraint: it is applied as PubMed's
upstream affiliation field and routes `--source all` to PubMed-capable search;
an explicit Semantic Scholar affiliation search fails before network work rather
than silently treating affiliation as free text or trusting sparse profile data.

The first useful release supports Semantic Scholar name search and exact detail
without `--affiliation`. Later tickets add PubMed affiliation-capable candidates,
ORCID detail/link evidence, and aggregates without changing the provider-
qualified ID grammar.

## Domain boundary

Add `src/entities/author/` as a sibling of `src/entities/article/`:

```text
src/cli/author/ -> src/entities/author/ -> src/sources/{semantic_scholar,pubmed,orcid}.rs
                                           |
                                           +-> shared publication identifiers,
                                               citation author/indexing values
src/render/{json,markdown/author}/ <- author result models
```

The author domain may consume publication identifier/byline/indexing values
factored into a lower-level shared model. `article` must not import `author`.
Source response structs remain source-local; only mapped author-domain values
reach renderers.

### Core values

```rust
enum AuthorIdProvider { SemanticScholar, Orcid }
enum EvidenceProvider { SemanticScholar, Orcid, PubMed }

struct ProviderAuthorId {
    provider: AuthorIdProvider,
    value: String,
}

enum AuthorIdentity {
    ExactProvider { id: ProviderAuthorId },
    Linked { requested: ProviderAuthorId, records: Vec<ProviderAuthorId>, links: Vec<IdentityLink> },
    Ambiguous { label: String, candidates: Vec<AuthorCandidateRef>, evidence: Vec<EvidenceRef> },
    NameOnly { display_name: String, query: AuthorNameQuery },
}

enum AuthorCandidateRef {
    Provider(ProviderAuthorId),
    NameEvidenceCluster { display_name: String, publications: Vec<PublicationRef> },
}

enum LinkReason {
    SharedOrcid,
    OrcidOnCitation,
    AlignedAuthorshipOnSharedPublication,
    SharedPublicationId, // candidate evidence unless the author occurrence is aligned
    NameAndAffiliation, // candidate evidence only; never sufficient alone for Linked
}

struct IdentityLink {
    left: ProviderAuthorId,
    right: ProviderAuthorId,
    reasons: Vec<LinkReason>,
    evidence: Vec<EvidenceRef>,
    decision: LinkDecision, // linked | candidate | conflict
}

enum TemporalAnchor {
    CitationPublished(Date),
    ValidityInterval { from: Option<Date>, to: Option<Date>, observed_at: DateTime },
    ObservedAt(DateTime),
}

struct AuthorAssertion<T> {
    value: T,
    source: EvidenceRef, // carries EvidenceProvider
    temporal: TemporalAnchor,
}
```

Do not emit a numeric confidence score. Deterministic reason categories and the
supporting records are inspectable; conflicts are not averaged away. A shared
paper ID alone is not link proof because two records may be coauthors. It becomes
`AlignedAuthorshipOnSharedPublication` only when the provider author occurrence
maps unambiguously to the same citation byline occurrence (provider author ID,
author order, and compatible name/ORCID evidence). Name and affiliation alone
can rank candidates but cannot produce `Linked`. Every affiliation assertion has
a non-optional `TemporalAnchor`; it is never an unlabeled current affiliation.

#### Link decision matrix

| Records | Evidence | Decision |
|---|---|---|
| S2 ↔ ORCID | The S2 record exposes the same checksummed ORCID | `linked` / `SharedOrcid` |
| S2 ↔ ORCID | An S2 author occurrence on a PMID/DOI aligns uniquely by author order and compatible name to the PubMed citation author carrying the requested ORCID | `linked` / `OrcidOnCitation` + `AlignedAuthorshipOnSharedPublication` |
| S2 ↔ S2 | Same ORCID on both records | `linked`; retain both provider records as a provider split |
| Any pair | Shared PMID/DOI without unique author-occurrence alignment; compatible name/affiliation only | `candidate`, never `linked` |
| Any pair | Different ORCIDs on the aligned author occurrence, incompatible bylines, or one provider record maps to two citation authors | `conflict` |
| PubMed name group ↔ provider record | Name/affiliation only | `candidate`; PubMed contributes evidence, not an identity ID |

Fixtures must include coauthors on the same paper, changed author order, missing
middle names, S2 split records, same-name homonyms, and conflicting ORCIDs. Any
case not accepted by the matrix defaults to `candidate`.

### Publication membership

```rust
enum MembershipKind {
    ProviderAuthorId, // paper returned from an exact provider author endpoint
    OrcidOnCitation,  // citation author ORCID exactly matches requested ORCID
    OrcidWorkAssertion,
    NameAndAffiliation,
    NameOnly,
}

struct PublicationMembership {
    publication: PublicationRef, // PMID/PMCID/DOI/S2 corpus ID when present
    author: AuthorIdentity,
    kind: MembershipKind,
    evidence: Vec<EvidenceRef>,
    included_in_aggregates: bool,
}
```

`ProviderAuthorId`, `OrcidOnCitation`, and public `OrcidWorkAssertion` are exact
within the named provider/assertion. `NameAndAffiliation` and `NameOnly` remain
candidates and default to `included_in_aggregates: false`. Fuzzy name matching
never replaces stronger available evidence.

## Response contracts

All JSON query responses use the existing evidence/next-command envelope. The
examples below show required domain fields; `_meta.evidence_urls`,
`_meta.next_commands`, and `_meta.source_status` remain required.

### Search

```bash
biomcp --json search author -q "Louis S Williams" --affiliation "Cleveland Clinic" --limit 5
```

```json
{
  "query": {"name":"Louis S Williams","affiliation":"Cleveland Clinic"},
  "providers": [{
    "source":"pubmed",
    "results": [{
      "identity":{"kind":"name_only","display_name":"Williams LS"},
      "display_name":"Williams LS",
      "name_variants":["Williams LS","Louis S Williams"],
      "affiliations":[{"value":"Cleveland Clinic","temporal":{"kind":"citation_published","value":"2026-05-01"},"source":{"provider":"pubmed","pmid":"42146891"}}],
      "match_evidence":[{"kind":"name_affiliation","source":"pubmed","pmid":"42146891"}],
      "warnings":["PubMed does not provide an author identifier; refine or select a provider record before get/publications"]
    }],
    "pagination":{"kind":"offset","offset":0,"limit":5,"next_offset":5,"total":null,"total_relation":"unknown"},
    "status":"available"
  }],
  "_meta":{"source_status":[],"evidence_urls":[],"next_commands":[]}
}
```

Search results are always provider buckets. A later linked detail may connect
them; search never collapses them merely because names match. `--offset` is
accepted only with an explicit single source. `--source all` returns the first
page of each capable provider; each next command selects one provider and its
own next offset. There is no federated offset or total.

### Detail

```bash
biomcp --json get author orcid:0000-0002-7433-2740
```

```json
{
  "identity":{"kind":"exact_provider","id":"orcid:0000-0002-7433-2740"},
  "display_name":"Atul Janardhan Butte",
  "name_variants":[{"value":"Atul Janardhan Butte","source":{"provider":"orcid","record":"0000-0002-7433-2740"}}],
  "affiliations":[{"value":"University of California San Francisco","temporal":{"kind":"observed_at","value":"2026-07-14T00:00:00Z"},"source":{"provider":"orcid","record":"0000-0002-7433-2740","put_code":"1576165"}}],
  "provider_records":[{"id":"orcid:0000-0002-7433-2740","status":"available"}],
  "conflicts":[],
  "_meta":{"source_status":[],"evidence_urls":[],"next_commands":["biomcp author publications orcid:0000-0002-7433-2740"]}
}
```

An exact provider get that has no proven cross-provider links uses
`identity.kind: "exact_provider"`; `linked` requires at least one accepted link.
A linked response names both records and its accepted evidence:

```json
{"identity":{"kind":"linked","requested":"orcid:0000-0002-7433-2740","records":["orcid:0000-0002-7433-2740","semanticscholar:1716151"],"links":[{"left":"orcid:0000-0002-7433-2740","right":"semanticscholar:1716151","decision":"linked","reasons":["orcid_on_citation","aligned_authorship_on_shared_publication"],"evidence":[{"provider":"pubmed","pmid":"41776157","author_position":3}]}]},"conflicts":[]}
```

A conflict keeps exact records separate:

```json
{"identity":{"kind":"exact_provider","id":"semanticscholar:example"},"conflicts":[{"decision":"conflict","reason":"different_orcid_on_aligned_author","left_orcid":"0000-0002-7433-2740","right_orcid":"0000-0000-0000-000X","evidence":[{"provider":"pubmed","pmid":"example"}]}]}
```

It does not fail or select one assertion silently.

### Publications

```bash
biomcp --json author publications semanticscholar:2269573451 --source all --limit 20
```

```json
{
  "author":{"kind":"exact_provider","id":"semanticscholar:2269573451"},
  "corpora":[
    {
      "source":"semantic_scholar",
      "status":"available",
      "items":[{"publication":{"pmid":"42146891","doi":"10.1002/jha2.70309","semantic_scholar_corpus_id":"288352248"},"membership":{"kind":"provider_author_id","evidence":[{"provider":"semantic_scholar","author_id":"2269573451"}],"included_in_aggregates":true}}],
      "pagination":{"kind":"offset","offset":0,"limit":20,"next_offset":20,"total":39,"total_relation":"provider_reported"}
    },
    {"source":"pubmed","status":"not_linked","items":[],"pagination":null}
  ],
  "combined_total":null,
  "coverage":{"complete":false,"notes":["Corpora are independently paged; totals are not summed"]},
  "_meta":{"source_status":[],"evidence_urls":[],"next_commands":[]}
}
```

There is no merged continuation token or authoritative union total. Each corpus
owns its provider offset/cursor and status. `--source auto` (the default) pages
the provider named by the requested ID. `--source all` returns independent
buckets. A caller continues one bucket with that provider and its pagination
value. For an S2 ID, PubMed is `not_linked` until 523 accepts a link; after an
accepted ORCID/aligned-authorship link, the PubMed bucket is enumerated by exact
ORCID-on-citation membership, not by fuzzy name search. ORCID works remain their
own assertion corpus. Deduplication by PMID, then DOI, is allowed only inside a
bounded aggregate operation and must retain all memberships.

### Coauthors

```bash
biomcp --json author coauthors semanticscholar:2269573451 --max-publications 100 --limit 20 --offset 0
```

```json
{
  "author":{"kind":"exact_provider","id":"semanticscholar:2269573451"},
  "coauthors":[{"identity":{"kind":"exact_provider","id":"semanticscholar:..."},"display_name":"...","publication_count":7,"supporting_publications":[{"pmid":"42146891"}]}],
  "corpus":{"source":"semantic_scholar","examined":39,"eligible":39,"excluded_uncertain":0,"truncated":false,"max_publications":100,"source_status":[]},
  "pagination":{"kind":"offset","offset":0,"limit":20,"next_offset":null,"total":null,"total_relation":"derived_bounded_corpus"},
  "_meta":{"source_status":[],"evidence_urls":[],"next_commands":[]}
}
```

Coauthors are counted only from aggregate-eligible memberships. Provider IDs
are preserved when present; a name-only coauthor is labeled `name_only` and is
not merged with another same-name coauthor.

### Topics

```bash
biomcp --json author topics orcid:0000-0002-7433-2740 --source pubmed --max-publications 100 --limit 20 --offset 0
```

```json
{
  "author":{"kind":"exact_provider","id":"orcid:0000-0002-7433-2740"},
  "topic_kind":"source_indexing_mesh",
  "topics":[{"descriptor":{"ui":"D001185","name":"Artificial Intelligence"},"publication_count":12,"major_topic_count":8,"supporting_publications":[{"pmid":"41776157"}]}],
  "corpus":{"source":"pubmed","examined":23,"eligible":21,"excluded_uncertain":2,"truncated":false,"max_publications":100,"source_status":[]},
  "pagination":{"kind":"offset","offset":0,"limit":20,"next_offset":null,"total":null,"total_relation":"derived_bounded_corpus"},
  "_meta":{"source_status":[],"evidence_urls":[],"next_commands":[]}
}
```

The first topic surface is deterministic MeSH aggregation and says so. Its
PubMed corpus is enumerated by exact ORCID-on-citation membership (or by a
previously accepted aligned provider link), not a fresh fuzzy name search. The
example's `eligible` records are `OrcidOnCitation`; `excluded_uncertain` records
are name/affiliation candidates and do not contribute. A future generated
summary must use a different `topic_kind`, retain its supporting publication
set, and ship under a separate quality contract.

### Markdown requirements

Markdown is not allowed to hide states carried by JSON. Stable headings/notes
must include these meanings (exact prose may evolve, meaning may not):

```text
## Author candidate — Williams LS
Identity: name-only (PubMed has no author ID)
Affiliation at publication (2026-05-01): Cleveland Clinic [PMID 42146891]
Membership: name + affiliation; excluded from aggregates

Warning: Multiple same-name candidates remain unresolved. Refine by affiliation
or select a provider-qualified record; BioMCP did not merge them.

Source status: semantic_scholar unavailable; PubMed results are partial coverage.
```

Exact/linked detail lists every provider record and accepted link reason;
conflicts have a separate `Conflicts` block. Publication/coauthor/topic Markdown
always prints corpus source, examined/eligible/excluded/truncated counts, and a
`Supporting publications` list or count with copyable identifiers.

## Source boundaries and operational contract

- `src/sources/semantic_scholar.rs` adds source-local author search, detail,
  batch, and papers request plans/models. S2 author IDs are provider evidence,
  not universal identity. Reuse its existing key-aware client and process-local
  limiter. Authenticated responses remain `NoStore` under current policy.
- `src/sources/pubmed.rs` reuses ESearch and citation EFetch. PubMed contributes
  name/affiliation candidate evidence and citation ORCID/MeSH assertions; it
  never invents a provider author ID.
- Add `src/sources/orcid.rs` for public-record and public-work request plans.
  Parse public fields only and preserve ORCID item source, put-code, visibility,
  and dates. Public email, homepage, inferred demographic data, and non-public
  assertions are excluded at the source DTO mapping boundary.
- All source legs use the entity's bounded outcome type and return
  `available`, `empty`, `degraded`, `unavailable`, `not_linked`, or
  `not_requested`. A failed optional source cannot turn into an apparently
  complete empty corpus.
- Shared HTTP cache policy remains the baseline for anonymous public requests;
  `--no-cache` and authenticated requests remain no-store. There is no durable
  identity graph or mutable profile cache in this target.
- Endpoint page and batch maxima are named constants in each source module and
  validated before requests. Source implementations may split a permitted
  request into provider-sized batches without hiding partial failures.

Concrete target limits:

| Operation | Public default / maximum | Provider boundary | Cache |
|---|---|---|---|
| S2 author search | 20 / 100 per provider bucket | API limit ≤1,000 and ≤10 MB response | anonymous shared cache, 24-hour max-stale fallback; authenticated `S2_API_KEY` requests `NoStore` |
| S2 author papers | 20 / 100 per page | API limit ≤1,000; offset continuation | same S2 policy |
| S2 author batch (internal) | n/a / 1,000 IDs | API maximum 1,000 IDs and ≤10 MB response; implementation may chunk lower | same S2 policy |
| PubMed author search | 20 / 100 per provider bucket | ESearch `retmax` accepts 1..10,000; author surface caps at 100 | anonymous shared cache, 24-hour max-stale fallback |
| PubMed citation hydration | internal chunks of 200 PMIDs | bounded EFetch chunks; failed chunks remain visible | anonymous shared cache, 24-hour max-stale fallback |
| Coauthor/topic input corpus | 100 / 500 publications | exactly one eligible source per operation in the first surface; no `all` allocation | inherited source policy |
| Derived rows | 20 / 100, offset 0 default | deterministic ordering after the bounded corpus is materialized | request-scoped derivation |

`--limit` on `search author --source all` applies independently to each first
provider bucket; continuation requires an explicit source. Aggregate v1 does
not accept `--source all`: coauthors use exact S2 author papers and topics use
an exact ORCID-on-citation/accepted-link PubMed corpus. This avoids an arbitrary
cross-source allocation policy.

Exact ORCID access mode, public defaults/maxima, practical anonymous ceilings,
redirect handling, terms, and cache freshness must be decided and pinned by 522
before 527 enables ORCID grammar. Until then, ORCID-shaped IDs are reserved but
rejected with an actionable unsupported-source error. If 522 cannot establish a
safe policy, 527/523 do not enable the network route and citation-supplied ORCID
remains evidence only.

## Invariants

1. No normalized name, fuzzy match, shared topic, or affiliation alone becomes
   a person identity.
2. External handles are always provider-qualified. BioMCP does not mint a
   global author ID.
3. Stronger membership evidence is never replaced by name-only matching;
   uncertain memberships are excluded from aggregates by default.
4. Provider disagreements and split/merged records remain visible with their
   evidence.
5. Publication corpora expose per-source status and continuation; no combined
   total implies completeness.
6. Every coauthor/topic claim retains the bounded corpus and supporting
   publications from which it was derived.
7. Affiliations are sourced and time-scoped; no historical citation affiliation
   is presented as current employment.
8. Topic output distinguishes source indexing from any future generated claim.
9. Source timeout/failure is visible in Markdown and JSON
   `_meta.source_status`; a healthy-source empty result is distinct from an
   unavailable source.
10. Only public professional identity evidence is mapped. Email, private
    profile fields, homepage, and inferred demographic data are absent from
    source DTOs, JSON, and Markdown.
11. CLI, list/help, MCP schemas/allowlists, docs, and executable specs move
    together for each newly public command.
12. Routine proof is fixture-backed; live identity traces belong in
    `make verify`, not `make lint`, `make test`, or `make spec`.

## Migration boundaries

The build order is deliberately additive:

1. repair bounded/status-bearing degradation on the existing article author
   route;
2. add S2 author source request contracts without public grammar;
3. ship exact S2 search/get, then exact S2 publications;
4. add internal PubMed candidate/membership mapping, then cut it over publicly;
5. prove the ORCID source, ship exact ORCID detail/publications, then expose
   evidence-bearing cross-provider links and conflicts;
6. derive S2 coauthors and PubMed MeSH topics from one bounded eligible corpus
   per operation.

Every stage leaves existing article and entity behavior intact. Existing
disease/variant article pivots continue to use the article domain and do not
inherit author identity or pagination semantics.
