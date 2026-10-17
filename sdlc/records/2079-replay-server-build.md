# 2079 build record — replay server and golden output tests

- Landed 2026-10-16 by direct merge 8756a7ea4 (pm's registry misfiled the ticket under 2003's branch, 2096/2078 precedent), changelog union resolved.
- Replay server serves recorded provider replies; 30 recordings manifest-wired; the golden suite rides pytest lanes behind the needs_binary marker; --update option registered at the tests/ root so CI's wholesale collection sees it.
- Four landing-CI false starts, each a separate mechanical cause: CI-only option registration (twice: wrong conftest level, then a dropped rebase commit), package-pin census drift, and a duplicate lint definition. Root lesson recorded: the boundary test's assertion is the only pin census; never precompute.
- Linux check host pre-push gates verified lint+test at the landing tip per the PM's confirmation-not-debugging rule.
- Later batches (the ticket's amendment) own the remaining command recordings; page conversions per 2080's handoff list.
