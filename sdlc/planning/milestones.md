# Milestones

Ian set these milestones on 2026-10-06 and can overturn any milestone, criterion, or placement.

BioMCP works on the 0.9 line. 1.0 is paused by Ian's direction. Every new ticket names its milestone, and 0.9 tickets are the default.

## 0.9.2

Outcome: the 0.9 branch queue landed on main, every external issue answered, the tag cut when Ian says.

Exit criteria: 1298 and 1294 land on a green merged-tree CI run, the yellow gates, and a fresh review verdict recorded on the ticket. The QA sign-off they first named is not owed: Ian waived it when he dissolved the QA team on 2026-10-06, before it answered. 1299, 1302, 1304, and 1291 land through the lane pipeline under the same bar. Issues #288 and #289 stay closed with fixes on main. Changelog bullets owed by the landed tickets are written before any tag.

## 0.9.3

Delivers: faster building and testing, with everyday checks on the team's own build machine and GitHub used only for releases; a setting that turns off individual sources and named groups of sources (issue #290); trustworthy variant literature search, supplements and PDF full text; and Windows cache-lock contention handled as a wait (issue #292), with a cache failure never making a provider look unavailable.

Serves: BioMCP's ideal state that every claim names its source, that a section is never quietly empty when a source failed, and that work stays bounded.

Done when: every 0.9.3 bug ticket has landed with its proof; the test and build speed tickets have landed; the final measurement compares 0.9.2 and the candidate on one build machine; a full manual release rehearsal passes on final main; and issues #290, #291 and #292 are answered with the shipped release.

## 0.9.4

Delivers: BioMCP's cache in one SQLite database, so lookups never scan the disk, size limits need no walk, and the CLI and the MCP server share it safely; golden output tests and the remaining fixture consumers on the replay server, with owned fixture identifiers; full FDA approval history for a drug; and, on top of that work, a fix for every known bug, the security updates, tests that fail only on product bugs, and process fixes that make the release path cheaper. `sdlc/plan.md` gives the order.

Serves: BioMCP's ideal state that work stays bounded and that the same binary runs safely as a shell command, a local MCP server and a shared HTTP server.

Done when: every 0.9.4 ticket lands or is withdrawn, main passes its full local check, the Linux fresh-clone offline run passes, and one full manual release rehearsal passes on final main without a tag.

## 0.9.5

Delivers: a measurement of whether an agent answers cancer care questions better with BioMCP; and proof that trial answers are correct against keys taken from ClinicalTrials.gov itself.

Serves: the two gaps BioMCP's ideal state names: nobody measures whether an agent succeeds, and nothing proves the trial answers are correct.

Done when: the agent measurement and the trial answer check each run and report a result, and a full release rehearsal passes.

## 1.0

Delivers: to be defined. BioMCP 1.0 is paused by Ian's direction, and this team continues on the 0.9 line.

Unknown: what 1.0 delivers beyond the 0.9 line, which work already done on the 1.0 branches carries over, and when the pause ends.

Break into tickets: when Ian lifts the pause, starting from a review of the 1.0 branches against the 0.9 line.
