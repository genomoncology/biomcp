# BioMCP backlog assessment — 2026-08-11

## Decision

Continue using focused, persistent-worktree sessions for groups of related
tickets. Preserve one commit and one completion record per behavior, but pay
the setup, architecture reading, fixture discovery, review, and complete-gate
cost once per coherent group rather than once per ticket.

The next focused session is the 14-ticket local-provider contract and
deterministic-fixture program described below. Do not use the SDLC queue for
that session.

## Backlog snapshot

At commit `3a9bc1fc`, BioMCP has:

- 81 files under `sdlc/tickets/`;
- 80 genuinely unfinished tickets;
- one stale completed ticket file, 0951, whose completion record already
  exists;
- zero drafts;
- zero missing dependencies; and
- 20 unfinished tickets whose prerequisites are already complete.

Remove the stale 0951 ticket file as record-keeping cleanup before the next
focused implementation merge. It is not unfinished product work.

## Consolidation assessment

Forty-seven of the 80 unfinished tickets fit eight coherent focused sessions:

| Focused session | Ticket count | Ticket IDs |
| --- | ---: | --- |
| Local provider contracts and deterministic fixtures | 14 | 0915, 0924, 0929, 0669, 0671, 0672, 0674, 0901–0907 |
| MCP execution and command catalog | 4 | 0919, 0930–0932 |
| CI and package guardrails | 4 | 0934, 0937, 0940, 0942 |
| Managed cache and session state | 4 | 0935, 0948, 0949, 0941 |
| Coordinate labeling | 3 | 0950, 0899, 0900 |
| Article retrieval and enrichment | 7 | 0909, 0876, 0877, 0882, 0926, 0682, 0684 |
| Installer and updater | 7 | 0910, 0911, 0916–0918, 0938, 0943 |
| Protein correctness and contracts | 4 | 0874, 0921, 0922, 0677 |

These sessions are dependency-closed in the order above. They share concrete
implementation and test seams rather than only similar titles. Completing all
eight would reduce the unfinished backlog from 80 to 33.

Of the remaining 33 tickets, 17 appear suitable for a later consolidation
wave: four ClinGen tickets, three machine-readable-output tickets, four
GWAS/schema/assembly tickets, four maintenance tickets, and the two final
offline-registry tickets. Eight release construction/signing/promotion tickets
should retain their separate trust boundaries. The remaining eight are broader
post-release usability or independent product changes.

## First focused session: 14 local provider contracts

### Objective

Make the selected provider-backed behavior deterministic and routinely
testable without public network access. Establish the shared production
transport proof once, reuse existing source planning and decoder tests, add
only missing execution/orchestration/rendering layers, and convert the owned
live specification blocks to receipted local fixtures.

Completion removes 14 unfinished tickets. Together with removal of the stale
0951 ticket file, `sdlc/tickets/` falls from 81 files to 66.

### Exact scope

Shared foundations and bounded provider behavior:

1. **0915 — Prove RequestPlan execution over a local transport.**
2. **0924 — Bound expanded UniProt and GTR data.**
3. **0929 — Reject unmapped NCI trial filters.**

Provider conversions:

4. **0669 — Convert pathway live assertions to deterministic ranking
   contracts.**
5. **0671 — Convert trial live assertions to deterministic CT.gov and NCI
   contracts.**
6. **0672 — Convert VAERS live assertions to deterministic aggregate
   contracts.**
7. **0674 — Convert diagnostic live assertions to deterministic source
   contracts.**
8. **0901 — Convert disease live assertions to receipted contracts.**
9. **0902 — Convert phenotype live assertions to receipted contracts.**
10. **0903 — Convert discover live assertions to receipted contracts.**
11. **0904 — Convert drug regulatory live assertions to receipted contracts.**
12. **0905 — Convert drug target and interaction live assertions to receipted
    contracts.**
13. **0906 — Convert gene identity live assertions to receipted contracts.**
14. **0907 — Convert gene enrichment live assertions to receipted contracts.**

### Why these belong together

Every conversion repeats the same chain:

1. parse the real CLI input;
2. construct the production `RequestPlan`;
3. observe that plan through a local transport;
4. decode provider-faithful recorded bytes with the production decoder;
5. run the production entity orchestration;
6. assert JSON and Markdown behavior; and
7. move the owned executable specification blocks into the routine registry.

Ticket 0915 supplies the common transport proof so each provider ticket does
not invent another HTTP test model. Tickets 0924 and 0929 repair the two
provider boundaries required by diagnostic/gene and trial conversion. Drug and
gene each have an intentional first/second slice that should stay in the same
worktree.

### Non-goals

- Do not change biomedical ranking, provider-selection, alias, pagination, or
  enrichment policy merely to fit a fixture.
- Do not replace production `RequestPlan` or decoder types with test-only
  projections.
- Do not weaken or delete a live assertion because a provider is unreliable.
- Do not perform the final all-provider live-registry reconciliation owned by
  0673.
- Do not add the fail-closed network namespace owned by 0884.
- Do not pull protein, variant-article, article-enrichment, or release work into
  this session.

## Execution plan

### 1. Establish the shared test boundary

Implement 0915 first with a reusable loopback fixture that observes the
production request and exercises the standard response reader and decoder.
Cover GET/POST, repeated query values, headers, supported bodies, redirects,
timeouts, body limits, status failures, malformed responses, and one complete
successful decode.

Run only the focused Rust transport tests until this boundary is green. Other
provider tickets cite and reuse it for generic execution behavior.

### 2. Inventory receipts and routes once

Before editing individual specifications, create one table of the required
provider routes and the real receipts already present in `testdata/` or fixture
directories. Reuse an eligible receipt instead of recapturing it.

For a missing real anchor, drive the production request path through the
recording seam and retain a dated request/response receipt. Hand-built URLs are
not eligible. Synthetic fixtures remain allowed only for edge states a real
provider cannot reliably produce, and must be labeled synthetic.

This inventory is working material for the session, not a second permanent
provider registry.

### 3. Repair prerequisite provider boundaries

Implement 0924 and 0929 next:

- 0924 uses small injected byte/row limits to prove exact-limit and
  limit-plus-one behavior for UniProt and GTR without constructing
  production-sized fixtures.
- 0929 uses a complete NCI filter table and a counting local transport to prove
  every unsupported value fails before a request and every supported value is
  mapped exactly once.

Keep these as separate commits and completion records because they change
runtime safety and filter behavior, not only fixture orchestration.

### 4. Convert paired provider families

Handle the two paired source families while their fixtures and models are
loaded:

1. 0904 then 0905 for drug regulatory, target, and interaction behavior.
2. 0906 then 0907 for gene identity and optional enrichment behavior.

For each pair, preserve the intentional temporary live-file split until its
second ticket moves the remaining blocks. Run the pair's focused source,
entity, fixture, renderer, and executable-page tests before moving on.

### 5. Convert the remaining independent sources

Complete 0674 and 0671 after their 0924/0929 prerequisites, then process 0669,
0672, 0901, 0902, and 0903. The exact order inside this final group may follow
fixture reuse and nearby code ownership, but each ticket retains its own red
test, implementation commit, focused proof, and completion record.

After every conversion, the Makefile, `scripts/run-specs.sh`, and architecture
inventory must agree on that intermediate routine/live set. Do not wait for
0673 to repair drift created by this session.

### 6. Review at two meaningful checkpoints

Perform one focused review after 0915, 0924, and 0929 to confirm the shared
transport and resource/filter boundaries are sound. Perform a second review
after all provider conversions to check:

- every claimed provider route has an eligible receipt;
- every fixture uses production request and decoder paths;
- unknown fixture routes fail closed;
- no converted routine path contacts a public provider;
- no assertion or provider behavior was weakened; and
- registries and documentation match the executable set.

Do not repeat the complete repository gates between individual tickets.

### 7. Record and seal

As each ticket becomes green, add its completion record and remove its ticket
file. Keep the commits independently readable and reversible even though the
work happens in one persistent branch and worktree.

On the final candidate run exactly once:

```text
make lint
make test
make spec
make full-feature-check
```

The ordinary gates must retain the full Rust, Python, documentation, and
executable-spec corpus. Run the distinct full-feature proof once because the
batch may change Rust provider code. Confirm the worktree is clean and no
fixture process, socket, temporary root, or server remains before merging the
whole branch.

## Done when

- All 14 scoped tickets have completion records and no runnable ticket copies.
- The stale completed 0951 ticket copy is removed.
- The shared transport contract is used instead of duplicated provider-generic
  executor tests.
- Every converted specification assertion runs through local provider-faithful
  bytes and production parsing/orchestration.
- Real anchors have request/response receipts; synthetic edge fixtures are
  labeled.
- The current routine/live registries agree everywhere they are projected.
- Focused tests and the one final sealed validation pass.
- The branch merges once, without resuming the BioMCP queue during the focused
  session.

## Decision after this session

Recount the backlog and inspect the diff concentration before selecting the
next group. The default next choice is the four-ticket MCP execution/catalog
session. If the provider work reveals substantial article-fixture reuse, the
seven-ticket article session may be more efficient immediately afterward.

## Implementation result

The 14-ticket session completed on 2026-08-11 in one persistent worktree.
Every scoped ticket now has a separate implementation commit and completion
record, and no scoped ticket remains runnable.

The consolidation produced three shared test boundaries:

- one production `RequestPlan` loopback transport contract;
- one supervised provider server for drug, gene, diagnostic, pathway, and NCI
  routes; and
- one supervised ontology server for disease, phenotype, and discover routes.

The existing CT.gov and VAERS servers were promoted to routine ownership.
Trial page blocks no longer start and stop CT.gov repeatedly. Unknown HTTP
routes fail closed. The receipt audit now recognizes 133 real, dated anchors;
synthetic data remains limited to three explicitly ineligible edge fixtures.

Nine formerly live provider pages now run routinely: drug, gene, disease,
phenotype, discover, diagnostic, VAERS, pathway, and trial. Their focused
executable pages passed 85 blocks in aggregate. The final batch review found
no added `src/` lines in the provider conversions; runtime source changes
were limited to the shared transport proof, bounded UniProt/GTR readers, and
the NCI filter correction.

Before the final repository gates, the top-level unfinished ticket count is
67 including the stale completed 0951 copy. Removing that stale copy leaves
66, exactly matching the planned reduction from 81.

The sealed candidate passed `make lint`, `make test`, `make spec`, and
`make full-feature-check`. Batch integration also removed shared-fixture
collisions for NIH Reporter and OpenFDA and serialized standalone CT.gov tests
against the routine fixture lock. The all-feature optimized build took 7m28s
from a cold release target; that release-only compile/link remains the largest
single measured build cost after the routine provider setup was consolidated.
