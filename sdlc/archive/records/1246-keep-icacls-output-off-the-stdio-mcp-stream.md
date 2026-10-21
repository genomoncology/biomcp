---
base: 09f6cafb
head: ba937a37
---

Kept icacls output off the stdio MCP stream on Windows, from GitHub
#283 via the review issue.

`icacls.exe` ran with inherited handles, so its localized success
line ("Successfully processed 1 files...") landed between JSON-RPC
frames on every managed write in stdio mode — GBK bytes on a zh-CN
console — and strict clients dropped the session. The child now runs
with stdin, stdout, and stderr all nulled: the protocol stream
carries frames only, and the protocol input pipe is no longer shared
either. Per-process memoization of successfully secured paths keeps
the repair posture (first encounter in a process still re-secures
pre-existing or externally broadened directories; failures retry) and
takes steady-state writes from roughly twenty icacls spawns per
`search gwas` call, the reporter's count, to zero.

A repository guard (`tests/test_source_child_stdio_guard.py`) fails
when non-test code under `src/` spawns a child whose stdout is unset
or `Stdio::inherit()` — only null, piped, or captured output passes,
allow-listing exactly the pass-through launcher. A Windows stdio
contract (`tests/windows_stdio_contract.rs`, run by the
`windows-contracts` CI job) spawns the real binary over stdio with a
fresh cache directory and a refused local provider port, drives an
HTTP-backed tool call whose shared-client construction secures the
tree before the fetch fails, and asserts every stdout line parses as
JSON-RPC with the call returning an isError result. Evidence
correction 2026-09-26: a scratch branch with the fix reverted
(9226cfe4, PR #285) proves the guard fails on the old code — CI run
36241795274's canonical-gates failed with all three streams unset at
src/cache/private.rs — but the windows-contracts job PASSED on that
same scratch: GitHub's runner icacls did not pollute the piped
stdout, so the behavioral failure of the old code is not
demonstrable on that runner and no such claim is made. The contract
stays as defense-in-depth for console sessions like the reporter's. The job also runs the memoization unit tests. The
changelog names the fix and #283 for the 0.9.1 release.

Evidence: design REJECT once (the guard trigger contradicted its
allow-list, the contract test had no offline mechanism, and the
remedy dropped the re-securing posture), folded and re-reviewed
ACCEPT; code review ACCEPT with two P2 notes (the contract child's
stderr now inherits so it can never fill a pipe; the guard's split
heuristic inherits the package-boundary pattern's edges and is
recorded); yellow gate at ba937a37 — lint, test, and spec OK after
one count-bump cycle (the boundary ratchet now counts the two new
test files, 1,356). CI on the merge then exposed that the lib's
test profile had never compiled on Windows: two unix-only test sites
(the article graph admission module and one provider-capture test)
were ungated, so the new `--lib` step broke; both are now
`#[cfg(unix)]` with the two pinned baselines taking the gate lines,
and main is green at a9b2d593.

Residuals: the before/after spawn count is the reporter's ~20 versus
zero by construction — both falsifiable in the Windows CI job, no
Windows host exists here; the in-process `SetSecurityInfo` directory
alternative is recorded as a follow-up only if memoization proves
insufficient in the field; the reply to the #283 reporter waits for
the release and Ian's OK.

## Guard follow-ups (2026-09-26)

The review's findings closed: the guard now requires all three
streams explicitly (null, piped, or captured) for every non-test
spawn — removing any single setter fails it, per-stream mutation
tests pin each; path-qualified and same-file aliased Command spawns
are caught; and the stripper no longer stops at the first mid-file
test marker — cfg(test) module blocks are brace-stripped and the
semicolon form `mod tests;` cuts at its own declaration, so the
seven render files whose production code sits after an early test
module are scanned (a fixture pins the shape; cross-module aliases
and brace-group imports are recorded as out of scope). The wheel
floor scan covers the biomcp-cli launcher by basename in any wheel
directory form, with a failsafe when nothing matches. The
fails-on-old-code evidence is real: scratch branch 9226cfe4 (PR
#285) reverted only the three setters and CI run 36241795274's
canonical-gates failed listing all three streams unset at
src/cache/private.rs; the windows-contracts job passed on that
scratch, because the runner's icacls did not pollute the piped
stdout, so the behavioral Windows failure is recorded as not
demonstrable on that runner. Code review REJECT once (the stripper
swallowed production code after `mod tests;`), fixed and re-verified;
yellow gate at 9a2ecd2a — lint, test, and spec OK.
