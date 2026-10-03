# Ticket 2009: Programme PM onboarding

Date: 2026-10-03.

- Ticket: [2009](../tickets/2009-onboard-programme-pm.md).

Administrative implementation is prepared on ticket 2009. Documentation preparation preceded ticket/design review. Root assigns a fresh ticket/design reviewer for the 2009 contract and records the actual review sequence. Separate fresh code review follows. Neither review is accepted here. Landing remains pending. The source programme target is `efff26d838ddc6471cfd9c06956855fd6e50ca43`. This record claims no product verification or completion.

## Release observation

GitHub release API reports [v0.9.1](https://github.com/genomoncology/biomcp/releases/tag/v0.9.1) published at `2026-10-02T02:08:04Z`, draft false, prerelease false and targetCommitish main. This observation establishes publication. It does not close migration tickets or prove their landing.

## Configuration decisions

See [the SDLC index](../README.md) and [the lane table](../planning/programme-pm-lanes.md). PM build `8592e26` supports the declared fields. Its ownership configuration protects matched files during automatic issue-closure link rewriting; it does not remove maintenance records from reports. The only status mapping is `done` to `complete`, supported by ticket 2007 and its recorded accepted product adoption. The new five-label Evidence format starts at ticket 2009. Earlier tickets retain their original contracts and proof records. Landing proof still starts at ticket 1. This adoption boundary removes format-only debt without declaring historical work complete.

## Verification

Initial normal PM reads covered all requested commands in human and JSON formats and item references 2001 through 2009. Committed-branch measurements follow below. Root owns external tool feedback. No hosted CI, full gates, providers, product changes or maintenance merges are authorized by this onboarding.

## Measured PM coverage

Measurements at administrative commit `1e89a7e53fc27a9cea9878bfa662f5256f51d0cf` used actual normal PM commands. Both human and JSON formats ran for status, next, daily, lanes and item references 2001 through 2009. The 26 invocations agreed on exits and represented states. JSON parsed successfully. Raw outputs stay local and unpushed. Final item verification exposed that the new numeric-only record title also failed PM linkage. The onboarding record now names Ticket 2009 explicitly so the active administrative item exposes it. This bounded correction leaves historical record titles untouched.

| View | Exit in both formats | Findings in JSON | Observed state |
| --- | --- | --- | --- |
| status | 1 | 185 | 1 in progress, 1 complete, 105 unknown tickets; 73 unknown issues |
| next | 0 | 182 | No ready candidates; retained merged gene branch awaits root cleanup; all six lanes occupied |
| daily | 0 | 182 | Programme target named; no recognized landings today; four inherited open review records; process check unavailable |
| lanes | 0 | 182 | Six existing worktrees; no missing worktrees or claim contradictions |
| item 2001 through 2006 | 0 | 182 each | Found with unknown status; coordinator decides |
| item 2007 | 0 | 182 | Complete; retained merged branch; root owns cleanup |
| item 2008 | 1 | 182 | Not found; branch exists without a ticket file |
| item 2009 | 0 | 182 | In progress; administrative implementation awaits fresh root review |

The common 182 findings comprise 105 ticket-status, 73 issue-status, two ticket-directory and two record-name findings. Status adds one duplicate ADR-number finding and two local-link findings. There are zero ticket-evidence findings after the explicitly adopted 2009 format boundary and zero missing landing-proof findings. This counts findings, not product readiness. PM does not schedule unknown tickets. Zero ready candidates does not mean the programme has no work.

Before configuration, normal status reported 1,559 findings and 106 unknown tickets. Numbered record naming, the documented done mapping and the new Evidence format boundary reduce format debt. They do not recover missing status or certify historical proof.

## Exact Git observations

| Lane | Observed HEAD | Commits absent from programme target | Dirty files | Unpushed |
| --- | --- | --- | --- | --- |
| programme | `efff26d838ddc6471cfd9c06956855fd6e50ca43` | 0 | 0 | 0 |
| gene-history | `9d8f072d48ed8d08589a765597faac815159c515` | 0 | 0 | 0 |
| maintenance-sync | `1dab73b270aeb082915f43bfb9186528516157b7` | 0 | 0 | unknown to PM |
| disease | `efff26d838ddc6471cfd9c06956855fd6e50ca43` | 0 | 0 | 0 |
| drug | `341e870dafc1cd182260aa8810395a3e0eb55045` | 56 | 0 | 0 |
| pm-onboarding | `1e89a7e53fc27a9cea9878bfa662f5256f51d0cf` | 1 | 0 | 0 |

Independent Git inspection matched all six branch names, target-relative counts and HEADs. Existing product lanes remain root-owned. Their path envelopes come from observed committed changes. A free path claim does not mean the retained worktree is available. PM next correctly marks each lane occupied by its branch. PM cannot check processes in this environment. It reports no running-agent evidence, not confirmed idleness.

## Mismatches and deferred interpretation

- Tickets 2001 and 2002 have implementation and verification records, but their ticket files lack a normal status line. Item reports unknown and finds no linked records. Their numbered record titles alone do not meet PM's title recognizer. Ticket 2002's historical hosted attempt failed before BioMCP execution. This onboarding does not fabricate final closure.
- Tickets 2003 through 2006 lack status lines. Historical review notes do not establish current assignment or completion. Root owns their disposition. This onboarding leaves them unknown.
- Ticket 2007 explicitly declares done and records accepted final product adoption. Its retained branch is an ancestor of the programme target. The done mapping yields complete with no landing-proof finding. Item nevertheless finds no linked records or review verdict. It also lacks an Outcome section to extract. These are PM coverage limits; product adoption evidence remains in the existing ticket and records.
- Ticket 2008 is absent from the target tree despite its retained branch. Its observed HEAD is already an ancestor of the programme target. The lane remains visible without manufacturing a ticket or closure.
- Next and daily exit 0 with 182 findings including unknown statuses. Their exit code is not a readiness gate. Daily includes inherited maintenance review records because ownedElsewhere does not filter reports.
- Two record-name findings concern `pending-review-check.md` and `review-fixes-for-1251-and-1252.md`. Archive and drafts directories produce two ticket-directory findings. The duplicate ADR number and local-link findings remain historical debt. Root already owns feedback for angle-bracket Markdown link parsing and exit semantics.
- `pm verify --clean` ran the configured light gates at local main `7099f253`, not programme target or onboarding HEAD. Both gates passed at that actual revision. The generated [receipt](../planning/checks.md) preserves its actual revision. It is not proof of this branch. Root owns feedback for target selection.

## Administrative checks and handoff

Direct `git diff --check`, `git diff --cached --check`, the March artifact rejection script and `python3 tools/check-tracked-text .` passed for the onboarding files. The tracked-text checker passed credential, placeholder, stale cache, deprecated public installation string and documentation code-leak checks. No product gate, hosted workflow or live provider ran. The per-command commit disabled repository hooks because adding PM JSON would otherwise invoke Rust formatting and clippy outside this authorized lightweight scope. The applicable lightweight checks ran directly.

The initial administrative commit was pushed with `[skip ci]` through the existing GitHub CLI credential helper. Fresh root review is pending. Record completion and landing only after root accepts the exact candidate and confirms the applicable proof. Historical tickets and product files remain byte-for-byte unchanged from the starting programme tree. Maintenance refs are not moved by this work. Ian can overturn the PM adoption boundary; root can revise working records using new evidence.

## Landed onboarding source

Fresh independent code review accepts `12a299a64620adbaff2019dbd8c55ecbdf0e0595`. Root adopted the reviewed documentation/configuration source by fast-forward and pushed it to `origin/biodata/biomcp-1.0` on October3. Candidate read checks and light documentation proof remain bound to that source. No runtime or hosted gate ran. Historical findings remain visible. This completion receipt updates delivery metadata only.
