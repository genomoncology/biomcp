# 2078 build record — article HTML onto scraper 0.27 and dom_smoothie

- Landed 2026-10-16 by direct merge b4a5ca89a after pm's ticket-branch registry misfiled the ticket under 2003's branch (2096 precedent; --record-only after manual merge).
- One HTML stack: scraper 0.27 + dom_smoothie replace the retired stack; 7 files, +221/-316.
- Gates: CI green at rebased tip 8a03a500a (the pre-rebase tip 4befb65d9 also green); Linux check host lint/test/spec green; the gencc size-baseline repin became obsolete at rebase (2076 deleted the inventory) and was dropped.
- Reviewed: accept at the rebased tip; lane report in the subagent record.
