# BioMCP 0.9 backlog recommendations

Written 2026-09-16 after the full 18-PR push, the saturated gate runs, and
the live CLI smoke test on both hosts. Organized by impact and effort. Each
ticket states what is wrong, the evidence, the proposed fix, and the
estimated complexity under the house rubric.

## Ticket A: Search trial --criteria returns zero results where --intervention finds trials

**What is wrong.** The `--criteria` flag on `biomcp search trial` filters
a narrower set than `--intervention` and returns zero silently. An agent
that gets zero results from a criteria search has no way to know the query
was valid but the filter was too narrow, versus the trials not existing.

**Evidence.** Filed as
`sdlc/issues/2026-09-11-search-trial-criteria-returns-zero-results.md`.
Reproduced during the CLI smoke test: `search trial -g BRAF` returned zero
while the same gene via `--intervention` found trials.

**Fix direction.** Either search the full criteria corpus instead of
filtering a pre-narrowed set, or return a hint when the result is zero and
the criteria path was taken. The issue file names both options.

**Estimated complexity.** Level 2 (Luna High). One source-layer change,
one outcome change, focused tests against existing fixtures.

## Ticket B: Citation-evidence gains a Crossref fallback provider

**What is wrong.** `article citation-evidence` covers two providers:
Semantic Scholar contexts (roughly 38 percent of edges have useful context)
and open-access Europe PMC JATS full text. When both are empty the command
returns `fulltext_unavailable` even though Crossref often has the citation
relationship and can supply the citing paper's reference list with the
cited DOI's position in it.

**Evidence.** During the live smoke test, a well-known pair (NEJM paper
citing a Nature paper) returned `fulltext_unavailable` because S2 had no
reference graph for the citing paper and the full text was paywalled. The
ticket-1145 research run found the same gap on 62 percent of edges.

**Fix direction.** Add a third state `context_from_crossref` to the frozen
five-state enum (making it six), reusing the exact-reference-identity rules
from the JATS extractor against Crossref's open citations API. The bounded
deadline architecture already supports multiple provider phases (ticket
1145's three-page graph deadline plus the bridge deadline pattern from
ticket 1191's fix). The JATS extractor's pure structural matching can be
reused directly.

**Estimated complexity.** Level 3 (Sol Medium). New source module, new
outcome state, renderer extension, acceptance tests across all six states.
Large but well-precedented.

## Ticket C: The citation sidecar makes evidence synthesis mechanical

**What is wrong.** Every call to `article citation-evidence` re-fetches and
re-parses. Agents that build on citation evidence across multiple queries
have no local record of what was already retrieved, so they re-request the
same edges and cannot accumulate a corpus.

**Evidence.** Filed as
`sdlc/issues/2026-08-27-a-citation-sidecar-would-make-synthesis-mechanical.md`
from an earlier research run.

**Fix direction.** A local cache of citation-evidence results keyed by the
directed edge (citing ID, cited ID), storing the outcome state, the
passages, and the retrieval timestamp. Read-before-fetch with a TTL. The
existing cache architecture (BIOMCP_CACHE_DIR) supports this; the citation
evidence command already has a frozen JSON shape that serializes directly.

**Estimated complexity.** Level 2 (Luna High). Cache layer on an existing
command with an existing cache infrastructure. Focused tests with
loopback fixtures.

## Ticket D: The discover command suggests a keyword search but cannot run one

**What is wrong.** When `discover` resolves no concepts from free text, it
helpfully says `Try: biomcp search article -k "..." --type review --limit 5`.
The agent must then execute that command separately. A `--search` flag
that runs the suggested command inline would save a round trip.

**Evidence.** Observed in the smoke test: `discover "vemurafenib
resistance"` resolved no concepts and printed the suggestion. The
suggestion's format is already a `NextCommand` with the exact CLI grammar.

**Fix direction.** Add `--search` to `discover` that, when no concepts
resolve, executes the suggested keyword search and returns its results
inline alongside the concept metadata. The next-command architecture
already builds the exact command string; the flag just runs it.

**Estimated complexity.** Level 1 (Luna High). One flag, one conditional
dispatch, tests with existing fixtures.

## Ticket E: Author search requires --query but bare-name drug search works

**What is wrong.** `biomcp search drug imatinib` works (bare-name
compatibility). `biomcp search author "Louis Williams"` fails with "Query
is required" because the author surface demands `-q` or `--query`. The
inconsistency is confusing for users who expect the same bare-name
convenience across entities.

**Evidence.** The CLI review's command matrix caught this as a smell
(`EXPECT 2 (author search requires -q/--query)`).

**Fix direction.** Add bare-name compatibility to `search author` the
same way `search drug` handles it: if the first positional argument is a
string without a flag, treat it as the query. Backward compatible because
the current behavior is an error.

**Estimated complexity.** Level 1 (Luna High). One argument parser
change, one test.

## Ticket F: Local-data sync commands could report per-source detail

**What is wrong.** The `--json` sync commands report `changed` (fixed in
ticket 1196) but not what changed. An agent syncing six sources cannot
tell whether the EMA bundle grew by one file or was completely replaced.

**Evidence.** The bundle fingerprint from ticket 1196 already computes the
sorted path/length/mtime triple before and after. The delta between the
two fingerprints is exactly the per-file detail, currently discarded.

**Fix direction.** Add a `changes` array to the sync outcome listing
added, removed, and modified paths computed from the fingerprint delta.
Backward compatible (new key, existing keys unchanged).

**Estimated complexity.** Level 1 (Luna High). Extend the existing
fingerprint comparison to emit the delta, one serializer change, tests.

## Ticket G: The ORCID surface could show claim counts in the header

**What is wrong.** `biomcp author papers orcid:<id>` returns the claimed
works but does not surface how many total works the ORCID record claims
versus how many were returned. A record with 200 claimed works and 10
returned (the default page) gives no signal that more exist.

**Evidence.** Not a defect — the pagination metadata does include
`next` — but the human-readable Markdown does not say "showing 10 of 200
claimed works" the way the trial surface says "showing N of M results."

**Fix direction.** Add the total claimed-works count to the ORCID record
fetch (the ORCID API already returns it in the works-summary group count)
and surface it in both the Markdown header and the JSON pagination object.

**Estimated complexity.** Level 1 (Luna High). One field extraction, one
renderer change, tests with existing fixtures.

## Ticket H: The MCP tool catalog could gain a discover tool

**What is wrong.** The seven-tool MCP catalog has no discover tool.
MCP-connected agents cannot use the concept mapper without falling back to
raw `biomcp` tool execution, which requires them to construct CLI
arguments rather than using a typed tool.

**Evidence.** The catalog test pins the seven-tool count and the
exclusion. The discover command has a stable, frozen output shape (JSON
with concepts and next_commands) that would map cleanly to a typed tool.

**Fix direction.** Add an eighth typed tool `discover` with a typed input
schema (query string) and the existing discover output shape. Update the
catalog pins, the measured byte/token ceilings, and the seven-tool tests
to eight.

**Estimated complexity.** Level 2 (Luna High). One tool definition, one
handler, catalog updates, tests. The hard part is the ceiling updates,
not the code.

## What I would not do

- **Do not add a summarization or interpretation layer.** The bounded,
  non-interpretive design is the product's strength. Agents need to trust
  what they get; they can do their own interpretation with an LLM they
  choose.
- **Do not add a federated meta-search that merges results across
  providers.** The exact-provider-record design is architecturally sound
  and the ADRs correctly avoid the identity-resolution swamp.
- **Do not chase the S2 author-papers zero-result case.** The provider
  has no data for that ID; no CLI change fixes a provider-side hole.
  The empty page with pagination metadata is the honest answer.

## Priority order

If forced to rank by user impact:

1. **A** (search trial criteria) — the most common silent-zero in daily use
2. **B** (citation-evidence crossref) — the biggest coverage gap in the newest feature
3. **C** (citation sidecar) — makes B's results accumulate across queries
4. **D** (discover --search) — saves a round trip on every unresolved concept query
5. **E** (author bare-name) — consistency fix, low cost
6. **F** (sync detail) — operational visibility
7. **G** (ORCID counts) — user-facing completeness
8. **H** (MCP discover tool) — convenience for MCP agents

A through C are the ones that change what users can actually accomplish.
D through H are quality-of-life improvements that make the existing
capabilities smoother.
