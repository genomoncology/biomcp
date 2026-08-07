---
base: 3710a5ea166d0dc2cef262a6ee7761d917868d94
head: a5c91d550dc74c99a0020eafbf16c4cb95d7ed0e
---
`biomcp search drug --region all` (default) returns a nested envelope (`us{count, results}, eu{count, results}, who{...}`) while `biomcp search drug --region eu` returns a flat `{pagination, count, results, _meta}` envelope. Scripts and agents navigating `search drug --json` must handle two structurally different shapes. Tracked as UX-4 since v0.8.20; not a regression, but a usability wart worth closing before v0.9.

Imported from March ticket 223. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/223-normalize-search-drug-json-envelope-across-region-modes-ux-4
