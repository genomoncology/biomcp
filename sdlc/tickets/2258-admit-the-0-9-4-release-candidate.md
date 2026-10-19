---
priority: 1
---

# 2258 — Admit the 0.9.4 release candidate

Status: OPEN.

Milestone: 0.9.4

Depends on: 2257, 2244, 2111, 2262, 2276, 2277, 2278, 2279, 2280, 2281, 2282, 2233, 2234

## Outcome

Main holds a stable 0.9.4 candidate that has passed every item in section 1 of `sdlc/release-checklist.md`, including a full untagged release rehearsal, and the PM has a go request naming the exact commit. No tag or publication.

## Order

1. After the last source landing, the admission commit lands through child ticket 2262: stable 0.9.4 in every public version file, the changelog section `## 0.9.4 — YYYY-MM-DD` covering every ticket since v0.9.3, and a disposition line for each finding in every open `sdlc/issues/` file this release touches (fixed with its commit, moot, or not a bug with the reason). `check-version-sync.sh`, `check-release-versions.py --tag v0.9.4` and `check-changelog-coverage.py --tag v0.9.4 --revision <sha>` pass without a tag.
2. On that candidate: the final 2233 Linux fresh-clone offline run and the 2234 paired measurement against v0.9.3, with raw evidence in a numbered experiment folder.
3. A fresh read-only reviewer reads every commit since v0.9.3 through the final tip, including records-only commits; commits after its accept get their own review.
4. The full untagged `release.yml` rehearsal on GitHub from the final commit with its full `candidate_sha` and `rehearsal_version` 0.9.4. Every required job succeeds. Record the run URL, the SHA and the unexercised publication limits.
5. Branch, worktree, reservation and scratch cleanup for this repository; `workspace repos --dirty` clean.

## Keeps

Work the checklist as written; copy it into the record and tick each item with evidence. A source or workflow change after the rehearsal needs a new rehearsal. Ian alone authorizes the tag.

## Proof

The release record lists every checklist item in section 1 with its evidence, and the go request goes to the PM with the commit, the rehearsal run, the gate evidence and the GitHub issues 0.9.4 fixes.

## Progress

- 2026-10-19 started
- 2026-10-19 next: Frozen source 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8 passes full main and live replay (39 checked/passed, eight explicitly unverified). Final Linux and paired measurements run before record review and full untagged rehearsal; no tag.
- 2026-10-19 next: Release admission held by final Linux concurrent cache-entry assertion;2283 is the sole repair lane. Main M5 full gate and live39/39 enabled coverage remain passed on51f5e03; candidate measurement arms await repaired source. No tag.
- 2026-10-19 next: PM approved test-only2283 under existing best-effort admission policy. Fresh review accepts; sole repair landing gate runs. Measurements resumed on unchanged product51f5e03. Final Linux whole run follows repair tip; live39/39 proof remains applicable to unchanged runtime/checker/recordings. No tag.

## Preparation evidence

Static preparation on 40c730fbe identified stable-version edits in Cargo.toml, Cargo.lock, pyproject.toml, uv.lock, manifest.json, server.json and CITATION.cff, plus the dated CHANGELOG section and current-candidate wording in docs/reference/release-process.md. Formula placeholders remain unchanged. No source edits or checks ran during preparation.

The active changelog lacks citations for landed 2191, 2202, 2203, 2204, 2205, 2210, 2211, 2213, 2214, 2228, 2231 and 2232. Describe each actual outcome, including retained profiles, rejected TOML trial, retained linking and metadata-recovery behavior; do not imply rejected alternatives shipped. Recompute final coverage after source and qualification records land.

Issue dispositions need finding-specific landing references for GTR encoding, transcript numbering, AlphaGenome CA bundles, trial-alias caching, CAID pagination, stale get JSON notes, removed lifecycle tests, aspirin identity, MCP probe counters, diagnostic synonym coverage, deadline joins and the single-CPU readiness handshake. The two October 7 omnibus reviews require individual historical dispositions; no blanket closure or new clinical judgment. The coordinator asked the PM to identify any still-material findings or approved deferrals.

## Final collection ownership

The PM assigned 2258 the mandatory observed drift collection on the final candidate. Ticket 2259 owns the accepted checker implementation and may close with its product batch. Before go, this ticket must retain the full collection report, refresh historical missing content types from the recomputed inventory of observed replies, refresh the stale dbSNP declaration from the current product endpoint, name all three KEGG recordings as not checked live: commercial-restricted source, and resolve every other drift or coverage gap through its owning fix. Never count KEGG as checked or call it live from these runs. Partial coverage is not a release pass.

## Shared replay ownership proof

Combined gate 2247/1792436359063-73075 passed at d836b94937f03ce9f35be43345af96a2f39c913b. Attached monitor 2260/1792436390360-77726 completed successfully: nine distinct servers across 829 samples, peak five total and at most one per direct owner, including the explicit lifetime probe. Observed owners covered pytest workers and runner, Rust test binaries and the specification shell; none were unclassified. Three final counts were zero. Sampling intervals averaged 0.414 seconds and ranged from 0.363 to 1.640 seconds, so shorter-lived processes can escape observation. Prior controlled shared-owner SIGKILL proof remains valid: two borrows reused one server and runner exit -9 removed the child and socket in 0.136 seconds. Raw first and repaired runs remain separately retained under experiment 804.

## Retained research replies

The release defect lanes retain a private, read-only copy of research runs 0031, 0032 and 0033. All 3967 files match the retained source SHA256 manifest. Source replies subject to licensing remain private; only permitted projections belong in public fixtures. This preserves lane inputs independently of the research store.

Keep: experiments/2081-biomcp-2258-research-fixture-retention — retain the private research replies and source checksum manifest used by the release defect lanes.

## Observed live collection scope

Changes: `testdata/replay/manifest.json`, `sdlc/records/2258-replay-live-collection.md`.

After stable metadata lands, recompute the actual recording inventory. Use observed credential-free primary replies to refresh historical missing response content types and the stale dbSNP request declaration, retaining original recording provenance and dates. Retain raw replies privately with exact request settings, status, case-insensitive Content-Type, observation time, bytes and SHA256. Final coverage uses tools/check-replay-live --live on the final candidate and records every entry and complete counts. No hardcoded historical inventory count.

Never call KEGG live; every KEGG recording says not checked live: commercial-restricted source and never counts as checked. Do not print, log, hash or store credentials or a derived value. Reuse authorized source settings; no new paid access. One serial request stream, bounded response size and time, no redirect/proxy/retry loop; obey Retry-After and stop the affected provider.

No checker/runtime/code changes in this collection. Do not publish broad or licensed source replies. Report every blocked, drifted or unsupported entry with the supported next repair or recording action; source/fixture body changes belong to a separately reviewed bounded fix. Partial coverage never means pass. Retain collection proof in a numbered private experiment with OWNER and Keep; the public record contains safe request metadata, counts, hashes and limitations. No tag or publication.

## Final qualification authority

The source and fixture batch is frozen at pushed main 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8, shared gate 2277/1792450592155-61493 passed with tree 90d9f94dfa5541a135bc728d88f849e595537d15. The actual-main full check is running before further ordinary landing. Use the accepted current checker for one final complete live inventory on this revision. Disable commercial-restricted sources and semantic-scholar explicitly. The three KEGG rows remain not checked live: commercial-restricted source. The PM approved Semantic Scholar as not checked live: rate limited after the one later 429 diagnostic; its five rows remain unverified, and no further requests to that source are authorized. Do not count any disabled entry as checked or passed. No credentials except an existing NCI_API_KEY may be inherited; never inspect, log, hash or store credentials. No new keys, proxies, authentication, paid access or provider contact.

The full untagged release rehearsal already includes MCP contract conformance; the PM withdrew its mistaken request for an additional separate conformance run. The existing checklist remains the release scope.

## Final live proof

Job 2258/1792451039742-39317 passed the actual frozen source 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8: 47 total, 39 checked, 39 passed, eight disabled, zero blocked, zero drift, zero ready. Complete 48-line output and settings are retained. KEGG rows14/15/16 remain commercially restricted; Semantic Scholar40/41/45/46/47 remain rate-limited and unverified under the PM ruling. Neither source was called. Preparation failure1792450988450-21123 made no calls; the final invocation used installed Python3.12.13. The safe collection record names shape-only limits and missing wire details.

Keep: experiments/2096-biomcp-2258-live-collection — retain original observed collection, source inventory, requests, raw hashes and repair provenance.

Keep: experiments/2102-biomcp-2258-final-live-replay — retain complete final per-entry checker output, exact candidate, settings, summary and preparation failure.

## Admission checklist

- [ ] Every open review file in `sdlc/issues/` for this release has a disposition line for each finding: fixed with its commit, or deferred with Ian's recorded approval.
- [ ] Every ticket merged since the last release has a ticket file, a code review verdict from a fresh read-only reviewer with its dispatch reference, and its local build-job gate evidence.
- [ ] A fresh read-only reviewer has reviewed every commit through the final tip, including records-only commits, and read the commit diffs. Commits after its ACCEPT have their own review.
- [ ] Every claimed fix fails its test when the fix is reverted.
- [ ] Every new or changed data source has its terms stated plainly in `docs/reference/sources.json` and `docs/reference/source-licensing.md`, including sources used only for identifiers.
- [ ] The changelog section `## X.Y.Z — YYYY-MM-DD` describes each change for users, names every GitHub issue the release fixes, and cites every ticket merged since the last release. `scripts/check-changelog-coverage.py --tag vX.Y.Z --revision <candidate-sha>` passes without creating a tag.
- [ ] Every public version file says X.Y.Z — the release commit drops the `-dev` suffix Cargo and Python carry between releases, and `check-release-versions.py` refuses the tag if any committed version still carries it. `scripts/check-version-sync.sh` and `scripts/check-release-versions.py --tag vX.Y.Z` pass against the committed stable candidate, without creating a tag.
- [ ] Every change landed through the coordinator's existing landing gate, singly or in a reviewed batch that shares one gate, followed by the full main check through a build job. The final candidate's routine, shipped-feature and required long checks passed.
- [ ] On the final candidate, run `python3 tools/check-replay-live --live` with the authorized source-disable and access settings. Retain every recording's result, the complete summary counts and candidate revision. Repair or re-record every blocked or drifted entry before tagging; name disabled sources explicitly because they remain unverified. See the release process for JSON/XML structure coverage and unsupported-format blockers and credential restrictions.
- [ ] Both the prior release and candidate measurements used the same build host, corrected measurement harness, toolchain, workload and cache settings; retain raw samples and label failed collections incomplete.
- [ ] Every `gh` call in `.github/workflows/release.yml` carries `GH_REPO: ${{ github.repository }}` (or a checkout), and every repository script runs after a checkout — `tests/test_release_workflow_provenance.py` enforces both. Every job that downloads the release while it is still a draft has `contents: write`.
- [ ] Merged ticket branches, worktrees, local branches, stale ticket-number reservations and scratch build folders for this repository are removed. Branches owned by the 1.0 team are left alone. `workspace repos --dirty` is clean for this repository.
- [ ] After every release ticket and the final measurement record has landed, manually run the full untagged `release.yml` rehearsal from the final candidate commit, with its full `candidate_sha` and stable `rehearsal_version`. Every required job, including `rehearsal-complete`, succeeds. Record the run URL and exact SHA in the go request. No tag or publication is created. A later source/workflow change requires a new full rehearsal.
- [ ] Record the rehearsal limits: GitHub release creation/upload/publication, PyPI trusted upload, registry authentication/pushes and `latest` movement, Pages deployment/live-site verification, and the Homebrew push remain unexercised. The rehearsal validates production builds, installed-package and container smokes, generated documentation and publication payloads without their credentials.

## Frozen-source gate evidence

Standalone checker gate 2281/1792449822402-39931 passed source tree 0229a6eb09dcfe6e84d16d5d44b54543cc8b9f68 and landed 6ddaed41268bfa49e992948673b8907de6798a33. Its actual-main full check 2156/1792450232278-98788 passed with the required all-feature and image proofs. The reviewed fixture batch then passed shared gate 2277/1792450592155-61493 at tree 90d9f94dfa5541a135bc728d88f849e595537d15 and landed frozen source 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8. Actual-main full check 2156/1792450891631-7897 passed in 405.149 seconds, including routine Rust, 1,575 Python cases, specifications, repository audit, all-feature checks and PNG/SVG/terminal output. Durable raw output remains in experiment 2101-biomcp-2282-pathway-recording-fidelity/main-checks.iFaHnu; shared gate raw remains in experiment 2097-biomcp-2277-canonical-disease-pages/batch-gate. Later commits are records only and receive their own review.
