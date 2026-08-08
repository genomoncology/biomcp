# Seven-variant recall canary fails all live coverage thresholds

Severity: should-fix.

This is live-verify-lane work. `make verify` is deliberately not a
gate rung here (`sdlc/planning/verify-lane.md`), so this cannot fail
a flight and should not outrank gate-lane work when it is triaged.

Carried over from March, where it was raised against ticket 605
on 2026-07-21 and left open. The text
below is as filed.
## Summary

`make verify` passed ticket 605's new G5 v2 identity canary at 7/7, but the older Seven-Variant Recall Canary failed every recall/coverage threshold. All seven legacy variants were reported incomplete.

## Detail

The failing verify run reported `reference_recall_at_least_9_of_12: false`, `variant_coverage_at_least_6_of_7: false`, and `route_specific_pmids_present_for_expected_variants: false`. `mlh1_family_pmids_present` remained true. The incomplete list was APC p.E1317Q, APC p.Q2322R, ATM p.C2464R, BRCA1 p.M1783I, MLH1 p.G67E, MSH2 p.L341P, and PTEN p.D326N.

The new authoritative-identity canary passed directly and inside the same live lane, so this is not evidence that ticket 605's RefSeq identity path is broken. It is an unresolved live-provider/recall failure in the pre-existing release canary. The current aggregate output does not expose enough per-route source status to identify the failing provider cheaply.

## Suggested action

Destination: **verify-group / experiment-harness**. Re-run the legacy canary in a credentialed release environment. If it remains red, capture each variant's route-level `source_status`, matched aliases, and terminal fields in the canary artifact, then repair the failing provider route rather than weakening PMID or coverage thresholds. Keep release promotion blocked until the existing canary is green or the provider failure is concretely triaged.
