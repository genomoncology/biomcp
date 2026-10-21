---
base: a7d503be
head: dbefb4d0
---

Closed the records cleanup from the 2026-09-27 review file, mostly
in documentation commits landed alongside the code tickets.

Both review issue files now carry item-by-item dispositions — every
finding marked Fixed (with its ticket), Deferred (with an issue), or
Open (with owner and trigger), nothing unmarked. The four files
without decisions got them: the DDInter Mac run (resolved by the
run itself), the one-CPU deadlock (stays open at P3 by recorded
choice, with owner and trigger), the 2026-09-26 review file
(disposition), and spec-contracts (decision plus the copied-workspace
tracking issue). Ticket 1253 carries the observed 26-bullet dry run
(1238 and 1243 added) and the #284 close after PyPI. The 1.0
candidates each name an owner and a revisit trigger in the backlog
doc; the variant transcript numbering failure became ticket 1260.
The contradictions (1254's three, 1242's gate line, 1243's stale
paragraph, the 1254 record's credentials overclaim) are corrected
to what the records and git history show. The two Status:open-over-
Resolved files and the yellow-lock wording (the lock lives in
dotfiles; BioMCP's ad-hoc gates ran on Ian's grant, not the lock)
now say what is true. The named shorthand ("the 1252 regime", "the
marked-watchdog form", "the 1242 render walk") is rewritten in
plain words.

Alongside, in the same window and recorded in their own files: the
Mac DDInter run (all three checks pass, numbers recorded), the
delegated licensing review pass (48 sources; COSMIC and CPIC
changed materially; Ian's COSMIC correction folded the same day),
and the DepMap decision with citations.
