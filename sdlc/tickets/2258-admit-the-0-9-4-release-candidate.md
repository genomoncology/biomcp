---
priority: 1
---

# 2258 — Admit the 0.9.4 release candidate

Status: OPEN.

Milestone: 0.9.4

Depends on: 2257, 2244, 2111, 2262

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

## Preparation evidence

Static preparation on 40c730fbe identified stable-version edits in Cargo.toml, Cargo.lock, pyproject.toml, uv.lock, manifest.json, server.json and CITATION.cff, plus the dated CHANGELOG section and current-candidate wording in docs/reference/release-process.md. Formula placeholders remain unchanged. No source edits or checks ran during preparation.

The active changelog lacks citations for landed 2191, 2202, 2203, 2204, 2205, 2210, 2211, 2213, 2214, 2228, 2231 and 2232. Describe each actual outcome, including retained profiles, rejected TOML trial, retained linking and metadata-recovery behavior; do not imply rejected alternatives shipped. Recompute final coverage after source and qualification records land.

Issue dispositions need finding-specific landing references for GTR encoding, transcript numbering, AlphaGenome CA bundles, trial-alias caching, CAID pagination, stale get JSON notes, removed lifecycle tests, aspirin identity, MCP probe counters, diagnostic synonym coverage, deadline joins and the single-CPU readiness handshake. The two October 7 omnibus reviews require individual historical dispositions; no blanket closure or new clinical judgment. The coordinator asked the PM to identify any still-material findings or approved deferrals.

## Final collection ownership

The PM assigned 2258 the mandatory observed drift collection on the final candidate. Ticket 2259 owns the accepted checker implementation and may close with its product batch. Before go, this ticket must retain the full collection report, refresh twenty-eight historical missing content types from observed replies, refresh the stale dbSNP declaration from the current product endpoint, name all three KEGG recordings as not checked live: commercial-restricted source, and resolve every other drift or coverage gap through its owning fix. Never count KEGG as checked or call it live from these runs. Partial coverage is not a release pass.

## Shared replay ownership proof

Combined gate 2247/1792436359063-73075 passed at d836b94937f03ce9f35be43345af96a2f39c913b. Attached monitor 2260/1792436390360-77726 completed successfully: nine distinct servers across 829 samples, peak five total and at most one per direct owner, including the explicit lifetime probe. Observed owners covered pytest workers and runner, Rust test binaries and the specification shell; none were unclassified. Three final counts were zero. Sampling intervals averaged 0.414 seconds and ranged from 0.363 to 1.640 seconds, so shorter-lived processes can escape observation. Prior controlled shared-owner SIGKILL proof remains valid: two borrows reused one server and runner exit -9 removed the child and socket in 0.136 seconds. Raw first and repaired runs remain separately retained under experiment 804.

## Retained research replies

The release defect lanes retain a private, read-only copy of research runs 0031, 0032 and 0033. All 3967 files match the retained source SHA256 manifest. Source replies subject to licensing remain private; only permitted projections belong in public fixtures. This preserves lane inputs independently of the research store.

Keep: experiments/2081-biomcp-2258-research-fixture-retention — retain the private research replies and source checksum manifest used by the release defect lanes.
