---
priority: 2
---

# 2321 — Routine tests warn on slowness and never fail on it

Status: OPEN.

Milestone: 0.9.4

## Outcome

BioMCP's routine gate fails on wrong behavior and on a hung test, never on a slow one. Per-test timeouts in nextest and pytest become hang catchers (for example 120 seconds), and test waits run up to them in place of short fixed limits. A small reporter prints one warning line for each test slower than 5 seconds, naming the test, its time and the machine's one-minute load, and changes no exit code. Product timing contracts that BioMCP promises, such as provider deadlines and the MCP handshake timer, stay as asserted bounds.

- Proof: a Rust test and a Python test made slow on purpose pass with their warning lines; a test made to hang fails at the hang catcher; the routine suite passes through pm job on the M5 and on Gator.

## Evidence

Jarvis asked for this on 2026-10-21 after pm's own root cause analysis found 8 of 17 failures came from test time limits alone; BioMCP hit the same class about ten times (2175, 2178, 2298 and others in root cause record 2299). It follows pm's ticket 0012 design and complements 2302 (event waits) and 2318 (no new fixed waits).
