# 2360 — Article full text stays bounded

Status: OPEN.

Milestone: 0.9.4

Priority: 1

## Outcome

`biomcp get article PMID fulltext` returns the full text, or a bounded failure that names each route it tried and why it failed, well inside an agent's tool deadline. It never runs past that deadline with empty output while a working route exists.

## Evidence

- Research recheck on the 0.9.4 candidate b56138d27 (practice release 39857815354), relayed by Jarvis in m3705: `biomcp get article 40966517 fulltext --json` on Blue, fresh isolated cache, keys set, produced empty stdout and was killed at the 120-second deadline (exit 124) twice, at 11:22:13 and 11:29:26 UTC.
- In the same run, Europe PMC PMC12448885/fullTextXML returned 200 with full XML in 1.3 s, so at least one route worked.
- Research claims no root cause or regression; Blue is shared and its load rose. Evidence on Blue: /home/ian/workspace/data/research/runs/0042/study-v1/cases/c001/ (calls.jsonl, call-0009, call-0011, call-0017). Keep key values out of any copy.

## Proof

- Reproduce first, on the candidate binary, with route-level timing: which route or wait consumed the time. Withdraw with evidence only if it does not reproduce and the timing shows plain host load.
- Outside-in test at the CLI: with one route stalled in a replay fixture and Europe PMC available, the command returns Europe PMC's text within the bound; with all routes stalled it fails within the bound and names the routes.
- Any fix needs 2330-style requalification and a complete untagged rehearsal on the new source.
