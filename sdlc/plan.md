# BioMCP plan

This plan describes future work and release exits. Read status from pm. BioMCP 0.9.3 is the only active release; work on the 1.0 line stays paused.

## Outcome

Ship 0.9.3 with correct product answers, a reliable local qualification process, a smaller useful test suite and measured results that compare like with like.

## Order

1. Repair main, then disable automatic branch, pull-request and main workflows before investigating historical failures. Preserve manual qualification and release publication. Use one coordinator for landings; stop previous landers and background work before takeover.
2. Install the existing job system's routine merged-candidate gate and postmerge standard/shipped-feature check. Include test code in routine lint. Land one reviewed ticket at a time and hold ordinary landings when main fails; reviewed repairs restore it before other work proceeds.
3. Prevent the small known failure classes: reject joined changelog entries (2158), derive package membership without an exact count (2159), and isolate provider-contract fixtures (2160). Reconcile ticket contracts and dependencies (2162).
4. Address research-raised product defects in dependency order: PubTator pacing (2149), full-text identity evidence under the existing identity rules (2139), source warnings and identity details (2088), batch PDF discovery pacing (2151), then bounded literal supplement searching (2152). Close the nine approved product gaps (2128) without inventing new clinical rules. Independent code may proceed in parallel where claims do not collide.
5. Fix coverage instrumentation (2120) before product-only coverage grouping (2123). Complete release-speed and cleanup work: active replay/golden replacements (2079) before each affected test deletion (2081), local parallel test lanes (2094), warning-only slow-test reporting (2095), required audit scheduling (2122), remaining fixture consumers (2111), benchmark/dead-command cleanup (2073) and dependency cleanup (2077). Serialize shared files through ticket prerequisites. Preserve each ticket's retained behavior and required outside-in checks.
6. Rework release qualification and add a complete untagged nonpublishing rehearsal (2161). Review public additions for private names (2129) after other measurement writers. Measure both v0.9.2 and the final candidate anew on the same build machine with the corrected harness and matching settings (2082), after every other implementation ticket.

Developers restate the outcome, reproduce the current behavior with an exact command and identify the failing outside-in check before code. Build jobs run on the designated build host. After automatic triggers are removed, checkpoint work according to team instructions. Review precedes the routine merged-candidate gate; the coordinator lands through pm and runs the full standard/shipped-feature check on actual main. Exhaustive suites run for a release candidate or changes to their covered behavior. Routine tests exceeding thirty seconds warn with load information; elapsed time alone does not fail them.

## Release exit

Every 0.9.3 ticket is landed or withdrawn through pm, main passes the required full local check, and comparable measurements include raw samples, method, immutable product/harness revisions and honest missed targets. After all ticket landings, including the measurement record, the complete manual release workflow must pass on final main without a tag or publication. Record that run through pm and chat without another provenance-only commit. Ian then creates the release tag; publication-only steps remain explicitly unexercised by rehearsal.

## Deferred work

The broad owned-identifier fixture refactor (2163) belongs to the next release unless a current fix requires a narrow repair. The 1.0 destination remains shared typed command values, with release-line changes flowing into that line when Ian resumes it. Do not use the paused line to change the active release.
