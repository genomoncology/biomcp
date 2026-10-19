---
priority: 1
---

# 2258 — Admit the 0.9.4 release candidate

Status: OPEN.

Milestone: 0.9.4

Depends on: 2257, 2244, 2111

## Outcome

Main holds a stable 0.9.4 candidate that has passed every item in section 1 of `sdlc/release-checklist.md`, including a full untagged release rehearsal, and the PM has a go request naming the exact commit. No tag or publication.

## Order

1. After the last source landing (2257, 2244 and the narrowed 2111 reader), one admission commit: stable 0.9.4 in every public version file, the changelog section `## 0.9.4 — YYYY-MM-DD` covering every ticket since v0.9.3, and a disposition line for each finding in every open `sdlc/issues/` file this release touches (fixed with its commit, moot, or not a bug with the reason). `check-version-sync.sh`, `check-release-versions.py --tag v0.9.4` and `check-changelog-coverage.py --tag v0.9.4 --revision <sha>` pass without a tag.
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
