# BioMCP plan

This plan says what BioMCP builds next, why, and in what order. Read status from pm. Work on the 1.0 line stays paused.

## 0.9.4: ship what we know is broken, fixed

1. Release blockers first: the security updates (2308 lock pins, 2309 rmcp 2.1) and the installed-wheel cleanup ownership (2310).
2. Test reliability from the root cause record 2299, so full checks fail only on product bugs: 2299 to 2304, the advisory-gate scope (2307) and the focused Python environment. Then add the checks that stop new fixed waits and paused clocks mixed with real sockets.
3. Process fixes from Ian's 2026-10-20 assessment that make the release path cheaper: one release document, release-only checks run earlier, the landing gate booking its real need, and removal of leftover pipeline scripts and re-stamping checkers.
4. Records cleanup (2306), which may follow the tag.

Exit: every 0.9.4 ticket lands or is withdrawn, main passes its full local check, the Linux fresh-clone offline run passes, and one full manual release rehearsal passes on final main without a tag. Ian then authorizes the tag.

## 0.9.5: prove BioMCP helps clinicians' agents

1. Full FDA approval history (2272).
2. Measure whether an agent answers cancer care questions better with BioMCP (ideal-state gap: nobody measures agent success), following `sdlc/planning/2026-10-03-prove-agent-value.md`.
3. Prove trial answers correct against keys taken from ClinicalTrials.gov itself (ideal-state gap: nothing proves the trial answers are correct).

Lanes run in parallel where their claims do not collide. Accepted tickets land in batches through one lander, each lane runs the routine suite on its branch merged with main before review, and the full main check runs once per batch.

## Hardest problems

These are the hardest problems BioMCP faces today, written for readers outside the team.

1. **Many outside sources, one answer.** BioMCP answers from dozens of public biomedical services with different speeds, rate limits and outages. A combined search waits for its slowest source, and one slow service can dominate the reply time.
2. **Naming variants without guessing.** The same protein change can be written several ways and numbered differently on different transcripts. BioMCP must match the forms a clinician means and refuse rather than return a confident wrong variant.
3. **Telling drugs, diseases and genes apart.** Brand names, combinations, biosimilars and ambiguous abbreviations such as MM or MF can point to the wrong record. BioMCP must pick the reading a clinician means or say why it cannot.
4. **Getting full text and supplements.** Many papers sit behind paywalls, publishers refuse scripted downloads, and old scans carry no text. BioMCP must find what is openly available and say plainly when it is not.
5. **A shared local cache that never breaks a request.** The cache must be fast and safe when many programs and a server use it at once on Windows, macOS and Linux, through upgrades, full disks and damaged files.
6. **Honoring source licenses.** Users must be able to switch off sources whose terms they cannot accept, including data those sources reach through other services, and BioMCP's records of each source's terms must stay accurate.
7. **Honest errors.** A user or AI agent needs to know whether a setting is wrong, a source is down, or the data does not exist, because each calls for a different next step.
8. **Testing against live services without flaky results.** Tests replay recorded provider replies offline, and any test that silently reaches the network or the tester's own files gives results that differ by machine.
9. **The same results on every platform.** Tests can pass on one operating system and fail on another, and routine testing on Windows is limited.
10. **Releases that pass on the first rehearsal.** Each full release rehearsal builds every package from a clean machine, and gaps in clean-machine setup have forced repeated rehearsals.
11. **Build time on a large codebase.** BioMCP has over 300,000 lines of Rust, nearly half of it tests, and development builds got slower in 0.9.3 even as tests got faster.
12. **Tests that prove what users run.** Some code paths are tested through test-only helpers instead of the code users run, and the suite must keep only tests that prove real behavior.
