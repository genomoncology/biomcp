# Raw ctgov-total MCP test SIGABRTs on the gate host only

Observed 2026-09-13 during ticket 1164's gate run.

`mcp::shell::tests::ticket_1120::raw_biomcp_tool_preserves_an_omitted_ctgov_total_as_null`
aborts with SIGABRT at about 1.4 seconds on the 4-core gate host, solo and
deterministically, three consecutive runs, both from the routine-test archive
and from a fresh cargo build, at ticket 1164's tip and at unmodified main
(a637e359). The same test passes on the 16-core dev host at main. This is a
pre-existing gate-host-specific failure, not a regression from any current
branch.

Worth considering: the test spawns the raw biomcp tool as a subprocess; the
abort smells like another host-flavor interaction (the gate host runs Ubuntu
25.10 with uutils coreutils, which already produced the timeout defect
recorded 2026-09-12). Capturing the abort backtrace on the gate host and
minimizing the spawned-command difference between the two hosts is the next
step for whoever takes it.
