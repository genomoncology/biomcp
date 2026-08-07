---
base: 5edf84f3300ff0a176ca6928ad2727eacfdd21ee
head: 58c3507f0c3a3cdf047e8dd973b6a454d12bed19
---
VarClassify's frozen G5 v2 panel now exercises BioMCP's exact-variant literature workflow, not merely recall. Five of seven identities resolve exactly, but two authoritative intronic variants do not:

Imported from March ticket 605. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/605-resolve-authoritative-refseq-assembly-identities-without-myvariant-records
