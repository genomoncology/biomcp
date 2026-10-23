# 2366 — Date-sorted search handles future issue dates and PubTator 400

Status: OPEN.

Milestone: 0.9.4

Priority: 1

## Outcome

A date-sorted article search orders by a real publication date and never puts future-dated issue placeholders ahead of published papers; the PubTator metadata request that returns HTTP 400 is fixed or no longer sent.

## Evidence

- Research recheck of the 0.9.4 candidate b56138d27 (research repo f1c305c, sdlc/records/0042-test-the-0-9-4-candidate-hard.md; raw outputs on Blue under /home/ian/workspace/data/research/runs/0042/), relayed by Jarvis in m3902. These are observations: reproduce on current main first, and withdraw with evidence if it does not reproduce.
- `biomcp --json search article -g BRCA1 --date-from 2026-09-01 --limit 50 --sort date`: PubTator metadata HTTP 400 persists (now disclosed as degraded), and the date-sorted page holds future issue dates such as 2026-12-31.

## Proof

- Outside-in replay test: a fixture with a future issue date and an earlier electronic publication date sorts by the electronic date; the PubTator request shape that drew the 400 is corrected and covered.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks this item on the final build.
