# 2051 — Answer pm and tidy the 0.9.2 records

Status: OPEN.

Milestone: 0.9.2

Reviews: revision 55b059c, accept

## Outcome

`pm daily` shows no unanswered asks, every 0.9.2 verdict names the tree that landed, the 0.9.2 tickets hold no hard wraps or status text, and no landed branch or worktree is left.

## Evidence

Filed 2026-10-11 from the independent review of main at `75fabdb20`.

- `pm daily` says "stalled: 5 asks from pm, pm PM unanswered 3 days". The answers were typed into the message files by hand (mailroom `d48a1ce`), so pm cannot read them. One answer says the lighter debug-info profile is "queued behind the 0.9.2 tag" and names no ticket.
- `sdlc/records/2048-pm-contract-build.md:3` says pm's asks are answered and the hard wraps fixed. Neither is true.
- Hard wraps remain in 2036 (lines 35-46), 2037, 2038, 2039, 2042 (lines 33-44), 2043, 2044 and 2045, and new wrapped text came in with the 2038, 2039, 2043, 2044 and 2045 sections in this range.
- `ac4c6739a`, a script edit 2048 named, reached main through merge `d47ce71ee`. Its "recorded through pm and normalized here" sections sit in 2037:52, 2038:155-171, 2042:89, 2043:59, 2044:61 and 2045:99. The pm line now carries each verdict.
- 2045 line 7 holds a stray "D" from the conflict resolution in `debce36c8`.
- The 2039 and 2040 verdicts name branch heads `ea5b22f5f` and `b6891d0f5`. The 2040 merge `0b59c8806` resolved a test conflict by hand and failed CI on main; `e6488f42c` fixed it. `239b16f5f`, 2048's contract-test fix, went onto main's first-parent line with no branch verdict.
- Status text sits in tickets: the 2039 and 2040 "Build status" sections, the 2046 "Candidate" section and `records/2038` ("rehearsal to follow").
- Landed remote branches remain: `tickets/2039-work`, `tickets/2040-work`, `tickets/2046-registry-derivation`, `tickets/landcheck-E` and the disposable `tickets/2039-probe`. Landed worktrees remain on the coordinator's machine: `biomcp-2039-lane`, `biomcp-2040-lane`, `biomcp-landE` and `biomcp-rehearsal-fix`, all clean with no unpushed commits, plus about 70 old local `tickets/*` branches.

- Starts from: main `75fabdb20` and ticket 2048.
- Keeps: the pm `Reviews:` line and the contract test that accepts it; record-only land commits.
- Changes: answer each ask with `pm answer`; file the debug-info profile as a ticket or answer that it is not wanted; correct the 2048 record; unwrap the 0.9.2 tickets and records in one commit; delete the hand-normalized verdict sections and the stray line; record landed-tree verdicts for 2039, 2040 and `239b16f5f` through `pm ticket review`; move status text out of the tickets; delete the landed branches and worktrees.
- Proof: `pm daily` and `pm inbox` show no asks from pm; the contract test passes; `git branch -r` and `git worktree list` show no landed 0.9.2 branch.
- Defers: nothing.
