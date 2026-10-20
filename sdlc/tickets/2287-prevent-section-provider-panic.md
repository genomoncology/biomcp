---
priority: 1
---

# 2287 — Prevent entity section provider-credit panics

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 6206f22, accept

## Outcome

`biomcp get protein P15056 structures --json` succeeds against the existing exact recorded UniProt response instead of panicking on its credited provider. Audit the same mismatch class across every entity section and fix any others found in this lane. Preserve truthful source credits and existing successful behavior.

Changes: `src/entities/source_state_registry.rs`, `architecture/technical/source-integration.md`, `tests/test_section_provider_credits.py`, `sdlc/records/2287-section-provider-panic.md`, `sdlc/tickets/2236-finish-the-remaining-replay-command-coverage.md`, `sdlc/tickets/2284-isolate-and-reduce-test-profile-rebuild-cost.md`, `sdlc/tickets/2288-*`, `sdlc/tickets/2289-*`, `sdlc/tickets/2290-*`, `sdlc/tickets/2291-*`, `CHANGELOG.md`.

## Evidence

Actual main e0d84c4293051b7f466bfcc58a68e2b739aace10, prepared CLI `biomcp 0.9.4+ge0d84c42`, diagnostic M5 job 2236/1792505124008-69036. Exact command `biomcp get protein P15056 structures --json` exits 1. Stderr says: `section outcome credits a provider outside the source-state registry: protein/structures`, at src/render/provenance.rs:120. The exact GET /uniprot/uniprotkb/P15056.json replay returned 200. Identity and structures Markdown succeed. The producer credits UniProt while the registry allows PDBe. The diagnostic script's exit 0 reports collection completion and is not a successful product command.

## PM decision

2026-10-20: Fix before 0.9.4 under Ian's every-known-bug rule. Add the outside-in JSON regression and one test walking every entity section's credited providers against the source-state registry. Fix any other mismatches found in the same lane. Compare 0.9.3 and record whether it had the panic, so the changelog is accurate. The later PM decision in 2236 moves replay expansion into 0.9.4 after the release-blocking repairs. Refresh only qualification invalidated by this repair, then run one full rehearsal.

## Boundaries

Read main and restate the goal, reproduce with the existing recorded input before code, and declare exact files and the failing outside-in check. No clinical interpretation, new captures, live calls, credentials or unrelated refactor. Preserve authored controls and the shared replay tools. The source-credit contract should have one owner; do not add a second maintained copy solely for proof. Developer proposes the bounded class check before implementation.

## Proof

Both the real CLI JSON regression and the bounded cross-section class check fail on current main and pass after the repair. Run affected tests and mechanical lint through pm job on the configured build host. Fresh review, landing gate and actual-main full check precede the new untagged rehearsal. Existing qualification and measurements remain bound to their actual source; do not claim this command was covered by the completed Linux run. No tag or publication.

## Progress

- 2026-10-20 started

## Bounded class prevention

Reuse the existing quality-ratchet registry parser and shared replay environment. Derive sections from the authoritative registry and exercise entity sections through the actual CLI against existing recorded inputs. Compare successful emitted provider credits with that registry; keep the explicit protein-structures JSON regression. The finite entity identifier inputs do not duplicate the provider mapping. Report unavailable outcomes as gaps in successful-provider coverage, inspect every production SectionOutcome constructor for that class, and disclose remaining gaps. Do not add captures or invent provider inventories. Preserve strict validation and correct the credited-provider declaration itself.

Keep: experiments/2102-biomcp-2258-final-live-replay — retain section-provider-repair/2287 raw preimage, corrected job output, exact commands and qualification identities until release admission closes.

Landing alone: this is the release-blocking product panic. Other accepted process repairs do not exist yet, and gate-definition changes must land separately. The PM record-only changes ride this repair as directed.
