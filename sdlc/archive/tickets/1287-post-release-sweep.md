# 1287 — the 0.9.1 post-release sweep

Filed 2026-10-02 from the sixth review file's sweep list, after v0.9.1 published green (release run 36952191952; the six issues #250, #282, #283, #284, #286, #287 closed with evidence from the published artifacts).

## What landed

- The checklist's rehearsal item now describes the rehearsal as it actually ran: the public scratch repository kept for the purpose, the stated differences (scratch image name, credential-free distribution check, stubbed docs gate), the rerun-not-retag continuation, the cleanup, and the runner rationale — private-repository runners are not available to this account, while public runners are free. Section 4 gains the item naming what the rehearsal's stubs skipped.
- The licensing evidence page drops the internal "go-request follow-up" phrasing, states the 48-versus-50 counts once, plainly, and explains why the Sanger DepMap Data Usage Policy covers Cell Model Passports (the portal documents itself under Sanger DepMap's documentation and links there for licensing).
- Record 1286 carries the Intel-runner retirement window (August 2027) and the Rosetta migration path.
- The DDInter judgment for the wheel-smoke sync, recorded here: DDInter's terms are CC BY-NC-SA 4.0 (academic, non-commercial, share-alike; the licensing entry carries the completeness disclaimer). The release workflow's smoke step downloads the bundle into an ephemeral runner environment to verify the wheel, uses it for nothing else, ships none of it, and the command remains `biomcp ddinter sync` for users — internal verification use, no redistribution, consistent with the 2026-09-27 review's handling of the source.
- The "P2"-style shorthand is removed from this cycle's round records — 1284, 1285, 1286 and 1221 (1221 rides in the 0.9.1 changelog, so it counts as this cycle) — findings are named in words. Records 0083, 0326, 0580 and 0592 predate this release cycle and keep their historical wording; rewriting them would rewrite what those reviews actually said.

## The three post-verdict commits

The sixth review verified the content of the three commits that landed after ticket 1284's verdict (the rehearsal-status merge 8af33dc9, the rehearsal-evidence merge d7176bce, and the corrections merge 992df4c8) in its GO review, and Ian checked 992df4c8 line by line himself before recording the GO at 3f696d63. This ticket records that coverage; the sweep branch itself carries this round's review dispatch below.

## Review

- Code review: ACCEPT 2026-10-02 dispatch 6158707d-b816-4906-b061-0c278967199e (Claude vendor, read-only), the branch's third pass — the first BLOCKed on the checklist edit that had landed outside the branch, the count contradiction and the shorthand scope; the second BLOCKed on the scope note still naming 1221 as historical and a duplicated checklist sentence; the third verified both fixes against the live files. Its one tooling gap (commit-range isolation) is closed by command: `git log --stat 13b48114..24dbd151` shows exactly the two named files.
