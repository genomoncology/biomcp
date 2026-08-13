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

## Next three focused sessions — decision recorded 2026-08-12

After the 14-ticket provider session and five-ticket MCP session, the active
backlog contains 61 genuinely unfinished tickets. No active ticket ID also has
a completion record. Ian approved the following next three focused sessions,
in this order. They contain 22 tickets and should reduce the active backlog
from 61 to 39 when all three are complete.

Do not reopen the batching analysis before each session. Revalidate that the
named ticket files and dependencies still match the repository, then use this
sequence unless intervening code changes create a concrete conflict.

### Session 1: genome identity and typed access — eight tickets

Tickets: **0927, 0928, 0950, 0899, 0900, 0933, 0690, and 0878**.

Recommended dependency order:

1. 0927 — make GWAS pagination fail safely.
2. 0928 — make GWAS filter semantics truthful.
3. 0950 — label the genome build on variant-detail JSON.
4. 0899 — carry genome-build labels through gene, search, and normalization
   JSON.
5. 0900 — name the genome build in every human coordinate rendering.
6. 0933 — generate entity-specific typed MCP search and get schemas from the
   authoritative catalog.
7. 0690 — prefer GRCh38 for ambiguous bare coordinates only after both the
   labels and typed assembly control exist.
8. 0878 — replace legacy population output with direct, explicit gnomAD v4
   results using a trustworthy GRCh38 coordinate.

The GWAS chain and coordinate-label chain can progress independently until
0933 and 0900 converge at 0690. Keep 0878 last so it consumes the settled
coordinate and assembly model instead of inventing another representation.

These tickets share genomic identity models, GWAS/MyVariant fixtures,
coordinate collision cases, JSON schemas, human renderers, command-catalog
entries, and typed MCP contracts. Doing them together lets required model and
constructor changes happen once. It also fixes the user-triggered GWAS abort
and false filter behavior before changing the preferred assembly.

### Session 2: installed-binary lifecycle — seven tickets

Tickets: **0910, 0911, 0916, 0917, 0918, 0938, and 0943**.

Recommended dependency order:

1. 0911 — establish one canonical fail-closed installer.
2. 0910 — let self-update read and verify current-sized release archives.
3. 0916 — establish the installer-owned binary receipt and refuse mutation of
   package-managed installations.
4. 0938 — make canonical installation a recoverable same-directory atomic
   transaction.
5. 0917 — reuse the ownership and transaction contract for durable Unix
   self-update; keep Windows self-update unsupported.
6. 0918 — make receipt-owned uninstall report partial or failed removal
   truthfully.
7. 0943 — remove shell-startup-file mutation after the canonical installer
   transaction is settled.

The common boundary is the complete lifecycle of one installer-owned binary:
download, checksum, ownership receipt, staging, smoke test, atomic replacement,
recovery, uninstall, and PATH guidance. The receipt schema, filesystem failure
fixtures, fake archive server, path/symlink defenses, and transaction-state
tests should be designed once and reused by the shell installer and Rust
commands. Keep separate commits and completion records for the seven behaviors.

This session is the strongest release-graph unlock after the genome work:
0911 and 0916 feed the updater, installer, packaging, candidate-staging, and
promotion chains.

### Session 3: article trust, retrieval, and receipted contracts — seven tickets

Tickets: **0909, 0876, 0877, 0882, 0926, 0682, and 0684**.

Recommended dependency and locality order:

1. 0909 — establish the safe provider-error logging projection first.
2. 0876 — classify Europe PMC's known not-open-access response as permanent
   absence rather than failure.
3. 0877 — preserve the typed non-retrievable article-asset outcome through
   direct lookup.
4. 0882 — preserve every complex JATS table cell in saved full text.
5. 0926 — make article candidate and enrichment source plans explicit and
   honor selected sources without hidden Semantic Scholar traffic.
6. 0682 — re-derive frozen variant-article identity from real receipted
   captures through production paths.
7. 0684 — move the seven-variant panel onto the captured routine corpus.

The first step protects every later provider failure path. Tickets 0876, 0877,
and 0882 share the PMID 30311380 Europe PMC/PMC/JATS corpus and full-text asset
orchestration. Ticket 0926 then settles actual article-provider planning before
0682 and 0684 freeze identity and panel behavior onto real receipts. Reuse the
existing supervised local transports, strict unknown-route rejection, article
output projections, and MCP single-execution boundary from the previous
sessions.

### Shared execution rules for all three sessions

- Use one persistent branch and worktree per session; do not use the paused
  SDLC queue.
- Preserve one red test, implementation commit, completion record, and removed
  active ticket file per behavior.
- Run focused tests while working and the complete repository gates once on
  each final session candidate.
- Do not weaken biomedical behavior to satisfy fixtures, and do not perform
  public-network work in routine gates.
- Recount the backlog after each merge. The expected counts are 53 after the
  eight-ticket session, 46 after the installer session, and 39 after the
  article session.

## Installed-binary lifecycle result — 2026-08-12

The seven-ticket session completed in one persistent worktree. Installation,
Unix update, and uninstall now share one receipt-based ownership contract with
unique same-directory staging, pre-commit version smoke, sync, pending-state
recovery, and one binary rename. Package-managed binaries are never mutated.
Windows self-update points to the installer. The checksum bypass and automatic
shell-startup edits were removed.

`make lint`, `make test` (2,875 Rust tests and 511 Python contracts), `make
spec`, and `make full-feature-check` passed. The cold all-feature release build
remained the dominant gate at 5m28s; focused lifecycle tests run in seconds once
their build profile exists. The active backlog is now 46.

## Article trust and retrieval result — 2026-08-12

The seven-ticket session completed in one persistent worktree. External
provider failures now have one credential-safe logging boundary. Europe PMC's
known non-open-access response is permanent absence, nonretrievable NCBI assets
have a typed browser fallback, and complex JATS tables preserve every source
cell with explicit merged-cell markers.

Article searches expose and honor separate candidate and enrichment source
plans, so an explicit provider selection makes no hidden Semantic Scholar
request. Variant-article identity now has a real receipted TP53 → CA000072 →
PMC8372092 table-annotation anchor through the production CAR and LDH paths.
The seven-variant 9-of-12 recall gate moved from credentialed public providers
to its real captured routine corpus, with exact-route refusal and production
JSON/Markdown checks.

All seven completion records are present and the active ticket files are gone.
The active backlog is now **39**, matching the planned 61 → 53 → 46 → 39
reduction across the three focused sessions. Final combined repository gates
are recorded in the batch integration commit.

## Runtime truth and deterministic protein result — 2026-08-12

The eight-ticket session completed in one persistent worktree: **0874, 0921,
0922, 0677, 0913, 0923, 0920, and 0673**. Variant coding and protein identities
now stay on one transcript; protein filters and pagination are independent and
truthful; and protein source behavior runs locally from real receipts.

Search-all counts distinguish exact totals from lower bounds. Batch commands
retain every successful item when another item fails. Every finite command
honors global JSON, while server commands reject it before startup. The
specification runner is now the only complete page registry consumed by Make
and verification.

`make lint`, `make test` (2,891 Rust tests, 512 Python contracts, and strict
documentation), `make spec`, and `make full-feature-check` passed. The active
backlog is now **31**, completing the planned **39 → 31** reduction.

## Remaining 31-ticket batching decision — 2026-08-12

Continue the direct, persistent-worktree approach. The remaining backlog is
not 31 independent flights: it is one pre-release closure session, one release
construction session, and three post-release substreams. Preserve individual
implementation commits and completion records, but pay the complete repository
gate cost once per focused session.

Only 0881, 0925, 0934, and 0935 are initially dependency-ready. The next
session deliberately starts with those prerequisites and continues through the
tickets they unlock without returning to the queue between them.

### Next session: pre-release closure — 12 tickets

Tickets: **0881, 0925, 0934, 0935, 0937, 0942, 0940, 0939, 0948, 0949, 0941,
and 0884**.

Recommended dependency order:

1. 0881 — preserve the real ERepo guideline identity.
2. 0925 — make PGx named sections focused and bounded.
3. 0934 — run canonical gates for pull requests and every main push.
4. 0935 — establish truthful session and HTTP-cache retention.
5. 0937 — enable the advertised PNG support in public artifact builds.
6. 0942 — add complete pinned shell and workflow linting to the canonical
   gate.
7. 0940 — exclude captured biomedical fixtures from source packages.
8. 0939 — replace the duplicate full `biomcp-cli` executable with the small
   compatibility shim.
9. 0948 — make managed cache and session state private.
10. 0949 — make `--no-cache` avoid all managed request state.
11. 0941 — make the Unix-socket test independent of ambient path length.
12. 0884 — enforce the final no-public-network boundary around routine gates.

This session has three internal work areas. Delivery and gate infrastructure
is owned by 0934, 0937, 0942, 0940, 0939, and 0884. Managed local-state
correctness is owned by 0935, 0948, 0949, and 0941. Tickets 0881 and 0925 are
the final two independent product-correctness blockers required before release
construction. They share little code, but keeping them in this session avoids
two extra setup and complete-gate cycles solely for singleton tickets.

Use focused tests after each behavior. Keep 0884 last so its isolated run is
the final proof that the resulting routine test and specification lanes make
no public connection. Run `make lint`, `make test`, `make spec`, and `make
full-feature-check` once on the final candidate. Expected backlog: **31 → 19**.

### Following session: release construction — seven tickets

Tickets: **0952, 0958, 0953, 0954, 0955, 0956, and 0957**.

Recommended dependency order:

1. 0952 — establish the pinned private candidate transaction with promotion
   still disabled.
2. 0958 — establish the protected signing and notarization seam.
3. 0953 — register and prove the five native and wheel targets.
4. 0954 — assemble the two-platform non-root container from verified Linux
   executables.
5. 0955 — generate and prove the Homebrew formula from the staged macOS
   artifacts.
6. 0956 — assemble and verify the declared MCPB desktop bundle.
7. 0957 — expose promotion only with complete candidate and public-artifact
   verification.

These tickets share one release workflow, candidate manifest, artifact
registry, signing policy, inspection framework, platform matrix, and
promotion transaction. Implement them in one persistent worktree, while
retaining their separate trust boundaries and completion records. Do not
publish, approve, tag, or move a mutable pointer during implementation.
External identities, protected-environment policy, hosted platform evidence,
and Ian's approval remain real requirements and must never be marked complete
from workflow text or local fixtures alone. Expected backlog: **19 → 12**.

Result on 2026-08-12: all seven tickets were implemented as separate commits
and completion records in one focused session. The repository now has a sealed
13-artifact private candidate transaction, protected signing policy, five
native/wheel targets, two-platform OCI image, generated Homebrew formula,
signed MCPB construction, and a protected public-proof promotion path. Local
fixtures exercised publication failure and replay without making public writes.
The complete lint, test, specification, and all-feature gates passed. The
committed signing policy remains intentionally disabled and unprovisioned; no
candidate was staged, approved, tagged, published, or promoted. Backlog:
**19 → 12**.

### Post-release substream 1: ClinGen completion — three tickets

Tickets: **0880, 0908, and 0962**.

Implement the CSpec attachment manifest, bounded ERepo gene search, and the
CSpec timeout/ERepo input boundaries together. They reuse the CSpec and ERepo
clients, real captures, request plans, local transports, typed limits, and
JSON/Markdown projections established by 0881. Expected backlog: **12 → 9**.

Result on 2026-08-13: all three tickets were implemented and closed with
separate completion records. CSpec now has explicit request deadlines and a
safe, metadata-only attachment manifest with exact capture reuse. ERepo batch
input is bounded before parsing, and gene assertion search has compact,
truthful continuation through both CLI and typed MCP. Real byte-faithful PTEN
captures and isolated fixtures cover both providers without routine public
network access. The seven-tool MCP catalog remains below 16,000 bytes and
4,000 tokens. CLI-only test modules moved under `tests/unit/cli`, preserving
coverage while leaving the combined production `src/` change at +58 net lines
against the batch's +520 allowance. Complete lint, **2,915 Rust tests**, **584
Python contracts**, strict documentation, executable specifications, and the
all-feature release smoke passed. Backlog: **12 → 9**.

### Post-release substream 2: bounded and relevant output — five tickets

Tickets: **0883, 0959, 0960, 0961, and 0963**.

These tickets share bounded response models, stable paging, continuation
commands, compact/full projections, typed source outcomes, local provider
fixtures, generated schemas, and renderer tests. They touch several entities,
so keep per-ticket commits and focused tests, but use one final complete gate
run. Expected backlog: **9 → 4**.

Result on 2026-08-13: all five tickets were implemented together and closed
with individual completion records. Entity responses no longer copy unrelated
executable workflows or make provider calls solely to choose guidance. Health
has exact provider selection, report-first failure exits, and consistent
six-state outcome language. Discovery, diagnostic results, drug regions, author
rows, article full text, and article asset manifests now have explicit bounds,
truthful totals, stable paging, and exact continuation commands.

The final review caught and repaired an important subtlety before closure: drug
ranking now classifies and orders the complete bounded provider result before
applying the user's per-region page, so an exact result on a later provider page
cannot remain behind an earlier broad match. Production `src/` grew by 1,054 net
lines against the batch's combined 1,200-line allowance; large CLI-only test
modules remain under `tests/unit/cli`. Canonical lint, **2,927 Rust tests**, **584
Python contracts**, strict documentation, the executable specifications, and
the all-feature release build passed. Backlog: **9 → 4**.

### Post-release substream 3: development and test maintenance — four tickets

Tickets: **0895, 0896, 0897, and 0965**.

Finish the tracked documentation-only hook, move the remaining setup fixtures
and run wrappers onto the generalized supervisor, and install the large-module
and dead-code ratchets. This work is intentionally last because it changes
development and test infrastructure rather than release behavior. The two
supervisor tickets retain their required order, 0896 then 0897. Expected
backlog: **4 → 0**.

### Shared rules for the remaining sessions

- Do not use the paused SDLC queue for these focused sessions.
- Keep one readable implementation commit and completion record per ticket.
- Remove a ticket file only after its behavior and dependencies are genuinely
  complete.
- Use red-green focused tests during implementation and complete repository
  gates once per final session candidate.
- Never weaken biomedical behavior, bounds, privacy, or release trust merely
  to make a fixture or batch pass.
- Reassess the post-release grouping only if the release implementation
  materially changes the named code boundaries; do not redo the batching
  analysis without concrete evidence of such a conflict.

## Pre-release closure result — 2026-08-12

The 12-ticket session completed in one persistent worktree: **0881, 0884,
0925, 0934, 0935, 0937, 0939, 0940, 0941, 0942, 0948, and 0949**. ERepo and
pharmacogenomics requests are truthful and bounded. Managed cache and session
state now has truthful retention, private permissions, and a real no-cache
path. The release surface ships PNG support without packaging captured
biomedical fixtures or a duplicate full executable.

CI delegates to the canonical gates, which now lint all tracked production
shell and workflow files. Routine tests and executable specifications run
inside a fail-closed network namespace after their artifacts are prepared.
That final isolation exposed and repaired stale MyGene, CPIC, ERepo, and build
profile fixture contracts instead of permitting public fallbacks.

Final validation passed: **2,904 Rust tests**, **533 Python contracts**, strict
documentation, the complete executable specification corpus, all lint and
quality checks, the all-feature release build, six AlphaGenome tests, and PNG,
SVG, and terminal artifact smoke. The active backlog is now **19**, completing
the planned **31 → 19** reduction. The next recorded session remains the
seven-ticket release-construction batch, beginning with 0952.
