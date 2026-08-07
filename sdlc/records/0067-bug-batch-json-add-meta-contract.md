---
base: 8c0d79aa06baea57bc6697450c157732383fb362
head: e985b2703f687dd7f05cc4e4f31df3daacf1460e
---
`biomcp batch gene BRAF,TP53 --json` returns a bare JSON array with no `_meta`:

Imported from March ticket 067. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/067-bug-batch-json-add-meta-contract
