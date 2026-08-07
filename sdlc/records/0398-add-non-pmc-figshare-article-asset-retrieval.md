---
base: 1ac71867e7961696b61cae02595e27ac0163b1c7
head: 347658a00c2215348a5db2371f23517c2c901a15
---
BioMCP 385 and 386 closed the PMC-backed part of article asset coverage: JATS now renders figure/supplement metadata when the XML carries it, and `get article <id> assets` / `asset <name>` can enumerate and stream files from a canonical PMC OA package.

Imported from March ticket 398. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/398-add-non-pmc-figshare-article-asset-retrieval
