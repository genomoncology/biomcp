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

## The ruling

Ian ruled on 2026-09-02: **refuse `active` as ambiguous.** Both sources, one behavior.

`--status active` returns a non-zero exit and an error naming the two unambiguous tokens. Write the error so a clinician can act on it without reading the source:

```
--status active is ambiguous: NCI uses "active" for a trial that is open and accruing, ClinicalTrials.gov uses it for one that has stopped accruing. Use --status recruiting for open and accruing, or --status active_not_recruiting for enrolled and no longer accruing.
```

The refusal applies to both sources, and it applies before any request goes out. Do not map the token on one source and refuse it on the other.

This breaks any script that passes `--status active` today. That cost is accepted. A token that silently returns the opposite of what a clinician asked for is worse than an error naming the two words that work, and 0.9 before a 1.0 rebuild is the cheapest moment this change will ever have.

Note the release: the change belongs in the changelog as a breaking CLI change, with the two replacement tokens named.

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
