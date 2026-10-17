# BioMCP plan

This plan describes future work and release exits. Read status from pm. BioMCP 0.9.3 is the only active release; work on the 1.0 line stays paused.

## Outcome

Ship 0.9.3 once every known bug is fixed, with faster tests and builds, reliable qualification, and measurements that compare like with like.

## Order

1. Repair main, then disable automatic branch, pull-request and main workflows before investigating historical failures. Preserve manual qualification and release publication. Use one coordinator for landings; stop previous landers and background work before takeover.
2. Install the existing job system's routine merged-candidate gate and postmerge standard/shipped-feature check. Include test code in routine lint. Land one reviewed ticket at a time and hold ordinary landings when main fails; reviewed repairs restore it before other work proceeds.
3. Prevent the small known failure classes: reject joined changelog entries (2158), derive package membership without an exact count (2159), isolate provider-contract fixtures (2160), and remove irrelevant randomized retries from the credential-redaction fixture (2165). Reconcile ticket contracts and dependencies (2162).
4. Address research-raised product defects in dependency order: PubTator pacing (2149), full-text identity evidence under the existing identity rules (2139), source warnings and identity details (2088), batch PDF discovery pacing (2151), then bounded literal supplement searching (2152). Close the nine approved product gaps (2128) without inventing new clinical rules. Independent code may proceed in parallel where claims do not collide.
5. Resume and re-review test cleanup (2081), then parallel test lanes (2094). Keep dead-command cleanup (2073) and audit scheduling (2122) in this release. Fix coverage instrumentation (2120) and warning-only slow-test reporting (2095); the warning work follows 2094. Retain the existing coverage groups and state their method and limitations; coverage regrouping (2123) is withdrawn.
6. Rework release qualification and add a complete untagged nonpublishing rehearsal (2161). Prepare consistent stable release metadata (2164). Review public additions for private names (2129) after other measurement writers. Measure both v0.9.2 and the final candidate anew on the same build machine with the corrected harness and matching settings (2082), after every other implementation ticket.

Developers restate the outcome, reproduce the current behavior with an exact command and identify the failing outside-in check before code. Build jobs run on the designated build host. Push the lane's branch when it is ready for review; do not push work in progress between fixes. Review precedes the routine merged-candidate gate; the coordinator lands through pm and runs the full standard/shipped-feature check on actual main. Exhaustive suites run for a release candidate or changes to their covered behavior. Routine tests exceeding thirty seconds warn with load information; elapsed time alone does not fail them.

## Release exit

Every 0.9.3 ticket is landed or withdrawn through pm, main passes the required full local check, and comparable measurements include raw samples, method, immutable product/harness revisions and honest missed targets. After all ticket landings, including the measurement record, the complete manual release workflow must pass on final main without a tag or publication. Record that run through pm and chat without another provenance-only commit. Ian then creates the release tag; publication-only steps remain explicitly unexercised by rehearsal.

## Deferred work

Release 0.9.4 contains dependency cleanup (2077), replay/golden expansion (2079), fixture-consumer migration (2111), and the broad fixture-identifier refactor (2163). Leave their lanes stopped and do not rebase them during 0.9.3 work. Make only a narrow fixture repair when it blocks an active bug fix.
