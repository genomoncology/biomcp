# 2363 — PubMed search answers within its deadline

Status: OPEN.

Milestone: 0.9.4

Priority: 1

## Outcome

Article search returns PubMed results when PubMed answers promptly, and reports a PubMed timeout only when PubMed itself is slow.

## Evidence

- Research recheck of the 0.9.4 candidate b56138d27 (research repo f1c305c, sdlc/records/0042-test-the-0-9-4-candidate-hard.md; raw outputs on Blue under /home/ian/workspace/data/research/runs/0042/), relayed by Jarvis in m3902. These are observations: reproduce on current main first, and withdraw with evidence if it does not reproduce.
- C003-PUBMED and C004-PUBMED: `biomcp search article -g CHEK2 --date-from 2026-09-01 --limit 50 --max-per-source 50 --sort date --json` and the CDKN2A c.149A>C query report "PubMed timed out after 12s" with complete=false, twice each. Direct ESearch from Blue returned 200 in 0.25 s, with and without the key. Find what in the PubMed path (follow-up efetch/esummary calls, rate pacing, queueing behind other sources) spends the 12 s.

## Proof

- Route-level timing names the cause. Outside-in replay test: a prompt PubMed fixture returns rows with complete=true under the same query shape.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks this item on the final build.
