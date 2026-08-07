---
base: b394979d8a1711a67fbe7ee6f06806fe889939e8
head: 101269ff7223309e141eca4389b037d2c637d50e
---
`biomcp get variant "chr10:g.87931071G>A"` returns `Error: API request to MyVariant.info failed. Retry the remote source.`

Imported from March ticket 685. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/685-declare-the-genome-build-on-every-myvariant-lookup
