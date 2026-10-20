---
priority: 1
---

# 2258 — Admit the 0.9.4 release candidate

Status: OPEN.

Milestone: 0.9.4

Depends on: 2257, 2244, 2111, 2262, 2276, 2277, 2278, 2279, 2280, 2281, 2282, 2283, 2285, 2233, 2234, 2286, 2287, 2288, 2289, 2290, 2291, 2236, 2284, 2292, 2293, 2294, 2295, 2296, 2297

Reviews: revision 1b8e728, accept

Reviews: revision fffcf09, accept

Reviews: revision 5db2cfd, accept

## Outcome

Main holds a stable 0.9.4 candidate that has passed every item in section 1 of `sdlc/release-checklist.md`, including a full untagged release rehearsal, and the PM has a go request naming the exact commit. No tag or publication.

Changes: `sdlc/records/2258-release-admission.md`, `sdlc/release-checklist.md`, `CHANGELOG.md`.

## Order

1. After the last source landing, the admission commit lands through child ticket 2262: stable 0.9.4 in every public version file, the changelog section `## 0.9.4 — YYYY-MM-DD` covering every ticket since v0.9.3, and a disposition line for each finding in every open `sdlc/issues/` file this release touches (fixed with its commit, moot, or not a bug with the reason). `check-version-sync.sh`, `check-release-versions.py --tag v0.9.4` and `check-changelog-coverage.py --tag v0.9.4 --revision <sha>` pass without a tag.
2. On that candidate: the final 2233 Linux fresh-clone offline run and the 2234 paired measurement against v0.9.3, with raw evidence in a numbered experiment folder.
3. A fresh read-only reviewer reads every commit since v0.9.3 through the final tip, including records-only commits; commits after its accept get their own review.
4. The full untagged `release.yml` rehearsal on GitHub from the final commit with its full `candidate_sha` and `rehearsal_version` 0.9.4. Every required job succeeds. Record the run URL, the SHA and the unexercised publication limits.
5. Branch, worktree, reservation and scratch cleanup for this repository; `git status --porcelain` empty on main and each owned worktree, plus `pm daily` cleanup evidence.

## Keeps

Work the checklist as written; copy it into the record and tick each item with evidence. A source or workflow change after the rehearsal needs a new rehearsal. Ian alone authorizes the tag.

## Proof

The release record lists every checklist item in section 1 with its evidence, and the go request goes to the PM with the commit, the rehearsal run, the gate evidence and the GitHub issues 0.9.4 fixes.

## Progress

- 2026-10-19 started
- 2026-10-19 next: Frozen source 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8 passes full main and live replay (39 checked/passed, eight explicitly unverified). Final Linux and paired measurements run before record review and full untagged rehearsal; no tag.
- 2026-10-19 next: Release admission held by final Linux concurrent cache-entry assertion;2283 is the sole repair lane. Main M5 full gate and live39/39 enabled coverage remain passed on51f5e03; candidate measurement arms await repaired source. No tag.
- 2026-10-19 next: PM approved test-only2283 under existing best-effort admission policy. Fresh review accepts; sole repair landing gate runs. Measurements resumed on unchanged product51f5e03. Final Linux whole run follows repair tip; live39/39 proof remains applicable to unchanged runtime/checker/recordings. No tag.
- 2026-10-19 next: Reviewed test-only2283 landed4c582db4186837893e4839f6d47e5e60a129aae7. Gate1792454342992-75134 and actualmainfull1792454758788-87321 passed; raw2103 retained. Whole final Linux rerun starts on this tip; paired measurements continue on PM-approved unchanged runtime51f5e03. No tag; final records and rehearsal follow.
- 2026-10-19 next: Final Linux 2233/1792455276832-365318 failed parallel article cache command1 during full Python; traceback pending, accessible raw evidence kept and caseDB teardown limit recorded. Sole repair2285 prepares diagnosis. Measurements continue old-release rows and hold before remaining candidate/runtime samples. Main M5 full remains green; no tag.
- 2026-10-19 next: Repair2285 landed as 40eac3b; actual-main M5 full job2285/1792458885162-87683 passed, raw retained in experiment2285. Linux renewal2233/1792458909149-436786 remains queued for memory under approved Rust reuse. Measurements complete; exact millisecond timing and fixed SQLite disk costs sent PM for disposition. Final records, fresh reviews and nonpublishing release rehearsal remain; no tag.
- 2026-10-20 next: Network restored. Final Linux2233/1792458909149-436786 independently verified PASS; fresh lane finishes retained record without rerun. Measurement2234 record accepted; interrupted landing resumed, new gate1792496574946-47864 runs. PM permits records-only receipt reuse and no separate full-main rerun; final rehearsal covers final tip. Fresh read-only admission reviewer /root/resume_release_audit_2258 active; no tag.
- 2026-10-20 next: 2233 and2234 records landed; exact candidatecffae4af5fb600039137b7c13d6955165ca46823/tree50e028fcb3dd0a17c0d5ee64e3678245895d6bc5 accepted through actualprefix. M5finalversion/changelog2258/1792497270549-70460 passed all86citations. Full nonpublishing release rehearsal39418376272 runs once on that SHA; main held fixed. Finalresult/cleanup/ownrecordreview follow, no tag.
- 2026-10-20 next: Fullrehearsal39418376272 failed bothmanylinuxinstalled-wheelsmokes after successfulqualification andall5archive/5wheelbuilds. Cache stats exits1aftervalidversion; causeunknown. Solefreshrepair2286 reproducesactualartifact/containerbeforefix. Admissionheld, no tag; preservefailedrunandreevaluateaffectedproofbindingsoncecauseknown.
- 2026-10-20 next: Reviewed wheel repair2286 passes both installed Linux smoke profiles; landing gate runs. Whole fresh Linux2258/1792502148739-726187 runs reviewed2ca without Rust reuse. Next actual-main full check and new full untagged rehearsal; original rehearsal remains failed, no tag.
- 2026-10-20 landed e0d84c4293051b7f466bfcc58a68e2b739aace10; next: Partial administrative records and citation landed. Finish repaired Linux qualification, then full untagged rehearsal and final admission review. No tag.
- 2026-10-20 next: WholeLinux2ca and actualmainc937full passed; administrativee0d and finalmetadata88 passed. Rehearsal held before dispatch because protein-structures JSON panics on currentmain. PM requires sole release repair2287 plus class check; refresh affected qualification then full untagged rehearsal, no tag.
- 2026-10-20 next: Main6e full passed; finish reviewed2288/2289/2290/2291 process batch, finite2236 replay batches and quietM5 rebuild2284 before affected qualification and one final untagged rehearsal.2272 accepted held for0.9.5. No tag.
- 2026-10-20 next: Main74d9f2d full407.756s passed.2288retrylandinggate underway;2290/2289 accepted integrated;2236partial57goldens accepted;2292CPICrepair live200s verified focusedchecksfinish. Quiet2284 timings then remaining2236 and finalqualification/rehearsal. No tag.
- 2026-10-20 next: Main e3db5a0 contains reviewedCPICrepair; gate327.945s passed, actualmainfull1792511348320-29396 runs. Main245 nativeinstalledwheel positive724.052s passed. Separate2289/2290 ready afterthismaincheck;2236 extends77acceptedgoldens withCCBY/OA3467229 fulltext/assets, onepermitted-minimizedmelanomacard and correctedCPICgoldens. Quiet2284 awaitsJarvis confirmation. No newrehearsal/tag.
- 2026-10-20 next: Main e3db5a0 actual full check1792511348320-29396 passed.2289 candidate gate failed two Python contract expectations; bounded repair lane owns reproduced failures, no helper changes.2290 latest-main integration a4a2e13fa freshly accepted, lands independently while2289 repairs.2293 format checker fresh accepted63edd277/source5e538,69checks; dependency merges into2236 before combined proof. OA primary8assets/fulltext captured; PM approved unavailable secondaryZIP aftertwo failed attempts.2284 prepares Gator, timings await quiet-hold relay. No new rehearsal/tag.
- 2026-10-20 next: Metadata2290 landed3386caf; actualfull1792513597082-84875 runs.2289 contract repair freshly accepted, currentcandidate5cfc0b524 carries PM2294 record into next permitted landing.2293 accepted and integrated with2236. PM added2294 product lane for primary assets avoiding optionalZIPwait; active separately from2236 coverage. Gator prepared;2284 timings await final process main and quiet hold. No new rehearsal/tag.
- 2026-10-20 next: Main65822d5 actualfull2289/1792514611103-53712 passed715.247s;2293format landinggate1792515409283-79476 runs.2295freshaccepted8a7/sourceaf473 adds fail-closed canonical policies and offline landing inventory.2236 accepted integratedce9:232checks retained, exact-sourceinventory107/97ready10disabled0blocked0checked passes.2294 fresh review owns a public collision-handle sufficiency fix; no runtime landed yet.2284 Gator prepared, awaits final reviewed runtime/main and PMquiet hold. Final rehearsal remains unstarted.
- 2026-10-20 next: Main175c004 contains2295; actualfull1792517340129-5977 runs.2236 latest-main integration38e has only reviewed SDLC status/markers beyond acceptedce9 and queues next.2294 reviewed148474 fixes scoped fixture lint;2296 focused14checks pass, mechanicals running.2284 formal Gator comparison runs against PM-approved3d baseline after exclusive hold; source/profile/prototype changes alone invalidate pairs. Finish accepted landings and final qualification before one full untagged rehearsal. No tag.
- 2026-10-20 next: Main175c004 full2295/1792517340129-5977 failed480615ms in installedLinuxwheel: E0463 find_msvc_tools missing throughcc/zstd-sys. Sole2297repair lane diagnoses source versus stalebuildstate; ordinarylandingsheld. Accepted2236gate runs,2294/2296queued.2284formalGatorpairscontinueisolated. No rehearsal/tag.
- 2026-10-20 next: Unchanged175 wheelretry2297/1792518079634-39053 passed373471ms; identicaldependencybytes, no sourcechange needed for recovery. PMdirected per-run cleantarget prevention2297 now develops with outside-in control. Mainhold stays through reviewedrepair/fullqualification.2236attributeexpectationfixed/accepted36570,focused4PASS;2294/2296acceptedqueued.2284finalGatorpairrunning. No rehearsal/tag.

## Preparation evidence

Static preparation on 40c730fbe identified stable-version edits in Cargo.toml, Cargo.lock, pyproject.toml, uv.lock, manifest.json, server.json and CITATION.cff, plus the dated CHANGELOG section and current-candidate wording in docs/reference/release-process.md. Formula placeholders remain unchanged. No source edits or checks ran during preparation.

Preparation found missing changelog citations for landed 2191, 2202, 2203, 2204, 2205, 2210, 2211, 2213, 2214, 2228, 2231 and 2232. Ticket 2262 supplied the reviewed citations and actual outcomes, including retained profiles, rejected TOML trial, retained linking and metadata-recovery behavior. Final coverage is checked again after qualification records land.

Issue dispositions need finding-specific landing references for GTR encoding, transcript numbering, AlphaGenome CA bundles, trial-alias caching, CAID pagination, stale get JSON notes, removed lifecycle tests, aspirin identity, MCP probe counters, diagnostic synonym coverage, deadline joins and the single-CPU readiness handshake. The two October 7 omnibus reviews require individual historical dispositions; no blanket closure or new clinical judgment. The coordinator asked the PM to identify any still-material findings or approved deferrals.

## Final collection ownership

The PM assigned 2258 the mandatory observed drift collection on the final candidate. Ticket 2259 owns the accepted checker implementation and may close with its product batch. Before go, this ticket must retain the full collection report, refresh historical missing content types from the recomputed inventory of observed replies, refresh the stale dbSNP declaration from the current product endpoint, name all three KEGG recordings as not checked live: commercial-restricted source, and resolve every other drift or coverage gap through its owning fix. Never count KEGG as checked or call it live from these runs. Partial coverage is not a release pass.

## Shared replay ownership proof

Combined gate 2247/1792436359063-73075 passed at d836b94937f03ce9f35be43345af96a2f39c913b. Attached monitor 2260/1792436390360-77726 completed successfully: nine distinct servers across 829 samples, peak five total and at most one per direct owner, including the explicit lifetime probe. Observed owners covered pytest workers and runner, Rust test binaries and the specification shell; none were unclassified. Three final counts were zero. Sampling intervals averaged 0.414 seconds and ranged from 0.363 to 1.640 seconds, so shorter-lived processes can escape observation. Prior controlled shared-owner SIGKILL proof remains valid: two borrows reused one server and runner exit -9 removed the child and socket in 0.136 seconds. Raw first and repaired runs remain separately retained under experiment 804.

## Retained research replies

The release defect lanes retain a private, read-only copy of research runs 0031, 0032 and 0033. All 3967 files match the retained source SHA256 manifest. Source replies subject to licensing remain private; only permitted projections belong in public fixtures. This preserves lane inputs independently of the research store.

Keep: experiments/2081-biomcp-2258-research-fixture-retention — retain the private research replies and source checksum manifest used by the release defect lanes.

## Observed live collection scope

After stable metadata lands, recompute the actual recording inventory. Use observed credential-free primary replies to refresh historical missing response content types and the stale dbSNP request declaration, retaining original recording provenance and dates. Retain raw replies privately with exact request settings, status, case-insensitive Content-Type, observation time, bytes and SHA256. Final coverage uses tools/check-replay-live --live on the final candidate and records every entry and complete counts. No hardcoded historical inventory count.

Never call KEGG live; every KEGG recording says not checked live: commercial-restricted source and never counts as checked. Do not print, log, hash or store credentials or a derived value. Reuse authorized source settings; no new paid access. One serial request stream, bounded response size and time, no redirect/proxy/retry loop; obey Retry-After and stop the affected provider.

No checker/runtime/code changes in this collection. Do not publish broad or licensed source replies. Report every blocked, drifted or unsupported entry with the supported next repair or recording action; source/fixture body changes belong to a separately reviewed bounded fix. Partial coverage never means pass. Retain collection proof in a numbered private experiment with OWNER and Keep; the public record contains safe request metadata, counts, hashes and limitations. No tag or publication.

## Final qualification authority

The accepted runtime and recording batch was published at main 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8, shared gate 2277/1792450592155-61493 passed with tree 90d9f94dfa5541a135bc728d88f849e595537d15. Its actual-main full check passed before the later test-only corrections. Use the accepted current checker for one final complete live inventory on this revision. Disable commercial-restricted sources and semantic-scholar explicitly. The three KEGG rows remain not checked live: commercial-restricted source. The PM approved Semantic Scholar as not checked live: rate limited after the one later 429 diagnostic; its five rows remain unverified, and no further requests to that source are authorized. Do not count any disabled entry as checked or passed. No credentials except an existing NCI_API_KEY may be inherited; never inspect, log, hash or store credentials. No new keys, proxies, authentication, paid access or provider contact.

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
- [ ] Merged ticket branches, worktrees, local branches, stale ticket-number reservations and scratch build folders for this repository are removed. Branches owned by the 1.0 team are left alone. `git status --porcelain` is empty on this repository's main checkout and each owned worktree, and `pm daily` confirms owned cleanup.
- [ ] After every release ticket and the final measurement record has landed, manually run the full untagged `release.yml` rehearsal from the final candidate commit, with its full `candidate_sha` and stable `rehearsal_version`. Every required job, including `rehearsal-complete`, succeeds. Record the run URL and exact SHA in the go request. No tag or publication is created. A later source/workflow change requires a new full rehearsal.
- [ ] Record the rehearsal limits: GitHub release creation/upload/publication, PyPI trusted upload, registry authentication/pushes and `latest` movement, Pages deployment/live-site verification, and the Homebrew push remain unexercised. The rehearsal validates production builds, installed-package and container smokes, generated documentation and publication payloads without their credentials.

## Frozen-source gate evidence

Standalone checker gate 2281/1792449822402-39931 passed source tree 0229a6eb09dcfe6e84d16d5d44b54543cc8b9f68 and landed 6ddaed41268bfa49e992948673b8907de6798a33. Its actual-main full check 2156/1792450232278-98788 passed with the required all-feature and image proofs. The reviewed fixture batch then passed shared gate 2277/1792450592155-61493 at tree 90d9f94dfa5541a135bc728d88f849e595537d15 and landed frozen source 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8. Actual-main full check 2156/1792450891631-7897 passed in 405.149 seconds, including routine Rust, 1,575 Python cases, specifications, repository audit, all-feature checks and PNG/SVG/terminal output. Durable raw output remains in experiment 2101-biomcp-2282-pathway-recording-fidelity/main-checks.iFaHnu; shared gate raw remains in experiment 2097-biomcp-2277-canonical-disease-pages/batch-gate. Test-only corrections 2283 and 2285 followed. Product Rust, Cargo files, replay recordings, the live checker and release workflow remain unchanged. Later record commits receive their own review.

## Admission review scope

Ongoing read-only review /root/review_release_commits_2258 read every commit diff through 51f5e03 and the owning late metadata deltas. It found no missing finding disposition in the reviewed issue scope, unsupported product-fix claim, source-terms gap, or missing ticket review/gate. It reused owning red/control evidence and supplied exact gate identities; it did not rerun every historical fix in reverse or reopen every private raw archive. Final admission must extend that review through later actual landings. The release has no newly fixed GitHub issue named in its changelog; earlier release issue numbers are not new 0.9.4 fixes.

The reviewed source terms include ORCID API/data distinctions, restricted identifier sources, and Reactome data versus artwork/software. Reviewed workflow provenance retains repository-qualified gh calls, checkout before scripts and draft-download permissions. The full nonpublishing rehearsal remains required.

## Cleanup evidence

The coordinator ran pm sweep --dry-run and removed only its fifteen explicitly abandoned empty reservation markers: 2064, 2069, 2070, 2075, 2083, 2085, 2086, 2108, 2118, 2119, 2121, 2123, 2135, 2153 and 2212. The push confirmed all fifteen deletions. Active release reservations, record-owned reservations, paused branches and unmerged historical work were retained. Current release worktrees and build output are removed after their consumers finish.

## Test-only qualification binding

Repair 2283 landed at 4c582db4186837893e4839f6d47e5e60a129aae7 and passed full actual-main job 2283/1792454758788-87321. Repair 2285 landed at 40eac3b38cf7d1f333f6bd34ee6119859547d500 after gate 2285/1792458419485-80284 passed. Its exact diff from 4c582db contains only tests/test_http_cache_fail_soft.py, CHANGELOG.md and the owned ticket/record Markdown. The coordinator independently verified that list and final test hash 2033f3ed0f93dba131fbd3630c55def78425fc53f00a8e97a88bc179449ef0de.

The PM approved reuse of the unchanged Rust routine/long/inventory proof while fresh Linux qualification rebuilds the exact CLI/example and reruns full Python, strict docs and shipped-feature checks under isolation. A Rust, Cargo, build, script or fixture-file change would require the whole Linux suite. Both failed final Linux attempts remain failed. The unchanged runtime measurement source remains 51f5e03 under the explicit reuse ruling; the live checker and recording inputs also remain identical to their 39-passed proof. These component bindings do not substitute for the required final rehearsal.

## Repaired wheel qualification

Repair 2286 is accepted at source 2ca44d1bd1430d7af2bb01bf98bb82d88aa8415c (runtime 9ac7829038d77c54dd86c864000ac393f2edf05d), with final evidence record 78e678ac58b96b87562a37d8d40e4310f9ff9c31 and separately reviewed retention metadata 2b7f09031. Both installed Linux wheels passed the PM-approved public smoke profile; live DDInter remains unverified. The first rehearsal remains failed. Whole fresh Linux qualification 2258/1792502148739-726187 runs the reviewed 2ca source without Rust reuse. Actual-main full checks and a new full untagged rehearsal follow landing. The PM approved reuse of the existing paired measurements and live checker evidence with this missing-directory correction disclosed.

Keep: experiments/493-biomcp-2258-repaired-linux — retain the exact runner, source identities, isolation verifier, full raw output, inventories and artifacts from repaired whole Linux qualification, including the prior attempt cancelled before any build or test. Regenerable clone and build copies may be removed after evidence is retained.

## Protein section release hold

Whole repaired Linux2258/1792502148739-726187 passed all stages on2ca, and actual repaired-main full2286/1792503071436-75490 passed on c937. Reviewed administrative partiallanding e0d and final metadata all88 citations passed. Before dispatch, 2236 reproduced a real protein-structures JSON panic on e0d. PM orders2287 release repair plus class prevention; no second rehearsal was dispatched. Refresh only qualification invalidated by the repair, then perform the full rehearsal. See the single admission record for all exact bindings and retained evidence.

## Final process batch

Ian moved 2288, 2289, 2290, 2291, 2236 and rebuild-cost work 2284 into 0.9.4. Finish these reviewed batches before the one new full untagged rehearsal. Formal rebuild measurements require a quiet M5. FDA approval-history feature 2272 stays accepted on its 0.9.5 branch until after the 0.9.4 tag. The earlier deferral of process improvements is superseded.

PM decision: current CPIC frequency and guideline routes return HTTP400 for obsolete sort columns. Repair 2292 is required for 0.9.4, including corrected public recordings and existing live-check coverage. The failed exact capture is retained under 2102 public-captures/2236; successful unrelated capture routes continue.

PM decision: bounded checker repair2293 qualifies real formats. XML DOCTYPE is accepted without entity expansion/external resource access; non-JSON/XML status/type/signature checks carry that explicit limited scope. Coverage reuses the unchanged39 prior live observations plus exact new observed captures and their CLI proof. No new full live collection.

PM decision: 2294 is required for 0.9.4. Complete sufficient primary archive asset answers return without waiting for the optional secondary ZIP; preserve fallback availability and budgets. The owning product lane is separate from 2236 recorded coverage.

PM decision: the rebuild comparison uses Gator for both arms after the PM relays its quiet-host hold. This supersedes the earlier M5-only measurement location for 2284; the completed 2234 M5 comparison remains historical context.

The bounded checker repair2295 resolves five existing canonical provider-policy lookup gaps without changing source permissions. Its inventory proof is required after integration; the prior232 recorded-command checks remain retained at their tested source.

PM decision: 2296 is a required release repair. Report stated licenses from fetched fulltext metadata and structured JATS without changing reuse policy or the recorded coverage scope. The isolated rebuild experiment uses approved published baseline 3d9472ecc2b4db2d68eee81d54c65453a9804806 on Gator. Relative before/after pairs need not use final main; only changes to Cargo, profiles or prototype code invalidate that comparison. Narrow license work proceeds independently.

PM decision: required repair2297 gives every installed-wheel run its own clean temporary build target after the unchanged-main recovery. This addresses the transient retained-build failure without changing the pinned toolchain, dependencies or installation smoke. Main remains held until reviewed recovery passes qualification.
