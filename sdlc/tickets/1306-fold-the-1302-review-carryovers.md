# 1306 — Fold the 1302 review carryovers

Status: complete.
Milestone: 0.9.2

## Build status

- Built on branch `tickets/1306-1302-review-carryovers`, sha `b835fc9ab`, 2026-10-07.
- Code review: ACCEPT 2026-10-07, no blocking findings. Two report-only P2s recorded: a PMCID seed's resolution search can still leave the degraded row reading not_requested (needs a flag from the shared resolver; deferred), and MalformedOci's status mapping is currently unreachable (harmless exhaustiveness).
- The citing seed is reused on the second-seed refusal (one fewer Europe PMC call, the richer seed record kept); the malformed-OCI wording no longer claims unavailability; the status row names the DOI-resolution phase per path. Tests fail on the old code on independent assertions.

## Outcome

## Outcome

Three honesty-polish items the 1302 review recorded but left out of the
fold scope land in one small pass.

## Evidence

Recorded as non-blocking notes in the 1302 review
(/tmp/review-1302-code.md mirrored in the ticket build status): the
second-seed refusal re-resolves both inputs through Europe PMC instead
of reusing the first seed's resolved pair; a shape-invalid OCI on a
matching row reports "was unavailable" when the index answered; and
_meta.source_status reports europe_pmc_jats not_requested while a Europe
PMC search may have run for DOI resolution.

## Change detail

1. Reuse the first seed's resolved citing side on the second-seed
   refusal path in `src/entities/article/graph/citation_evidence.rs`.
2. Split the shape-invalid-OCI wording from the unavailable wording.
3. Name the DOI-resolution phase in the status row.

## Keeps

- No behavior change to the normal pipeline or the degraded answers.

## Proof

The 1302 case-table tests extended for each wording; no new fixtures.

## Priority note

P3: polish; no user-visible defect today.

Note: hand-numbered after pm ticket new drew 2010 again (see the report
to the pm team, 2026-10-06, second occurrence with both seams declared).
