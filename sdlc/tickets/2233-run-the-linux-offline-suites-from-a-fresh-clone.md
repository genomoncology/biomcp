---
priority: 2
---

# 2233 — Run the Linux offline suites from a fresh clone

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `sdlc/records/2233-linux-offline-suites.md`, `CHANGELOG.md`.

The full Rust and Python suites pass on the Linux test machine from a fresh clone of current main with the network off, before any 0.9.4 release rehearsal, and each gap found has its own ticket.

## Evidence

The plan requires this check before any rehearsal. Clean-machine setup gaps forced repeated 0.9.3 rehearsals. Running it now, while 2134 and the replay work land, finds gaps before the release instead of during it.

## Keeps

Change no product code in this ticket. Record each failure with its cause and file a fix ticket for it. Use only the Linux test machine and remove the clone afterwards.

## Proof

One record lists the commit, the commands, the pass and fail counts and each failure's cause. Rerun once on the final release candidate after 2134 and the replay work land.

## Development brief

Run the declared full offline Rust and Python suites on the configured Linux test machine through a build job, using a fresh clone of current main 096c11fc416ddc18ee3b05814a6d0b9fede63648. Separate dependency preparation from actual network-isolated execution, prove kernel isolation and record exact commands, graphs, counts and exclusions. Collect failures without fixing product code. Report each distinct gap and its cause to the coordinator, who files its ticket. Remove only owned clone/build scratch after preserving evidence. This is the initial check; final candidate rerun remains required after cache and replay land. PM explicitly authorized this occasional Linux qualification.

## Progress

- 2026-10-19 started
- 2026-10-19 next: Initial fresh-clone offline qualification passed on096c11fc416ddc18ee3b05814a6d0b9fede63648 with no demonstrated gaps. Preserve evidence; rerun once on the final candidate after2134 and replay work land.
- 2026-10-19 next: Final frozen-source Linux offline qualification is running at 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8 in job 1792451222629-265544; retain initial and final evidence separately.
- 2026-10-19 next: Final Linux job1792451222629-265544 failed only concurrent CLI/MCP cache-entry proof on51f5e03. Rust4279passed; Python1587passed/1failed/5skipped; docs/full-feature notrun. Evidence170/final retained and scratch removed. Repair2283 owns diagnosis; rerun final qualification after reviewed repair.
- 2026-10-19 next: Test-only2283 landed actualmain4c582db4186837893e4839f6d47e5e60a129aae7 with passing landing gate1792454342992-75134. Fresh experimenter prepared whole final Linux rerun and failed-case temp retention; starts after actualmain full check. Preserve initial and failed evidence170.
- 2026-10-19 next: Whole final Linux job1792455276832-365318 submitted on4c582db after mainfullpassed. It waits for6GiB memory behind ThinkThen0619 booking13.5GiB; no build started. Resource route sentPM. Preserve queue and original170 evidence; no admission bypass.
- 2026-10-19 next: Final Linux job1792455276832-365318 admitted on exact4c582db. Dependency bootstrap completed and fresh builds/tests now run under existing kernel network isolation. Retain new evidence170/final-1792455276832-365318 separately; preserve failed-case temp if any stage fails.
- 2026-10-19 next: Renewed final Linux job1792455276832-365318 passed routine Rust4279 and long/inventory, then failed parallel article cache command1 at about39 percent of Python. Full Python continues for trace without retry. Sole repair2285 owns diagnosis. Accessible replay evidence retained; original caseDB already removed by TemporaryDirectory teardown. Stop after failed stage; no docs/fullfeature or record edits.
- 2026-10-19 next: 2285 accepted at 3e6575b; repair landing gate runs. PM permits reuse of Rust routine/long/inventory from 4c582db only if repaired-tip diff contains Python tests and records, with no Rust/Cargo/build/script/fixture-file change. Fresh clone, exact rebuilt CLI, complete corrected Python, strict docs and shipped-feature checks under isolation remain required. Fresh experimenter is prepared; release after actual main publication and M5 full start.
- 2026-10-19 next: 2285 landed main 40eac3b38cf7d1f333f6bd34ee6119859547d500 after passing gate 1792458419485-80284. Independent path check confirms only the HTTP Python test and change records; PM-approved Rust proof reuse applies. M5 full started and fresh Yellow qualification is released in parallel for exact CLI/example, full Python, strict docs and shipped features under isolation.
- 2026-10-19 next: Fresh qualification 1792458909149-436786 is queued on exact main 40eac3b under approved Rust-proof reuse. It requests 6 GiB; Yellow reports about 10.7 GiB available while ThinkThen 0619 books 13.5 GiB. No build started. Renewed resource route sent PM. Full M5 main check runs in parallel; no admission bypass.

## Final source assignment

Run the final fresh-clone offline qualification on frozen pushed source 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8 after the replay fixture batch. Preserve the earlier initial snapshot as historical proof; final evidence belongs in a separate subdirectory of the existing qualifying-host experiment. Reuse the established preparation, kernel isolation and command method, with two Rust workers and serial Python. No product changes, new harness or provider calls. If a gap appears, stop with its exact output and separate repair owner. Only the owned fresh clone and build scratch may be removed after evidence retention.

Keep: experiments/170-biomcp-2233-linux-offline-evidence — retain initial and final offline qualification logs, commands, source identities, isolation proof and artifacts.
