---
priority: 1
---

# 2264 — Report disease name lookup failures as unavailable

Status: OPEN.

Milestone: 0.9.4

## Outcome

A network or provider failure during `get disease --name` returns an unavailable or upstream-error outcome naming the source, never not_found, so a failure is never read as absence.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R1: `HTTPS_PROXY=http://127.0.0.1:9 biomcp --json get disease --name "Turcot syndrome" phenotypes` exits 1 with a 203-byte not_found and empty stderr, byte-identical to a failed live run; a normal run resolves MONDO:0010159.

## Keeps

A real no-match stays not_found. Check the other `--name` resolvers for the same fault and fix them in the same way.

## Proof

Replay tests: a failing provider gives the unavailable outcome; a true no-match gives not_found.
