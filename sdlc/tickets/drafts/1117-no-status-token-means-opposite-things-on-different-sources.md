---
flow: build
priority: 8
---

# `--status active` asks for open trials and returns closed ones

`normalization.rs:43` folds the token:

```rust
"ACTIVE_NOT_RECRUITING" | "ACTIVE" => Ok("ACTIVE_NOT_RECRUITING"),
```

and `nci.rs:105` turns that into an NCI request parameter:

```rust
"ACTIVE_NOT_RECRUITING" => NciStatusFilter::SiteRecruitmentStatus("CLOSED_TO_ACCRUAL".into())
```

So a user who writes `--status active` against NCI receives trials closed to accrual. NCI's own vocabulary uses `Active` for a trial that is open and accruing, which is the opposite. The word is a reasonable thing for a clinician to type and it returns the reverse of what they asked for.

Verified in `src/entities/trial/search/normalization.rs` and `src/entities/trial/search/nci.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Why this is a draft

The fix requires a ruling Ian owns, because both answers cost something outward-facing.

**Refuse `active` as ambiguous**, with an error naming `recruiting` and `active_not_recruiting`, applied to both sources. This is BioData's recommendation. The cost is that a working input becomes an error, so any script passing `--status active` breaks.

**Map `active` to NCI's meaning.** Nothing breaks today. The cost is that one token keeps meaning opposite things depending on which source answered, which relocates the trap rather than removing it.

My recommendation is to refuse. A token that silently means the opposite of what a clinician intends is worse than an error that names the two unambiguous words, and 0.9 before a 1.0 rebuild is the cheapest moment this change will ever have. The cost is real and it is a public CLI contract change, which is why it is Ian's call and not mine.

Whichever way it goes, write the decision into this ticket before promoting it, because the agent cannot ask.

## Required behavior

No status token means one thing on one source and the opposite on another.

## Done, observably

- `--status active` does not return trials closed to accrual.
- Every status token selects the equivalent set of trials whichever source answered, or is refused on both.

## Where correct behavior is written

`repos/biodata/sdlc/planning/clinical-trial-conformance/cases.json`, case 5. Take the assertion from that case, write it as a failing test, then fix. Do not copy the expected behavior into this repository as a second statement of it.

Reported by the BioData lead in `notes/biomcp/feedback/2026-09-02-seventeen-trial-defects-to-fix-in-0-9.md`, defect 5 of seventeen.
## Boundary

Do not change `recruiting` or `active_not_recruiting`. Do not change how a trial's status is displayed.
