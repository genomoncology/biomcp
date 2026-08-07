---
base: 662066f305217f0e7bf758cceb14ae9d2930da8c
head: 1d593e2b80157b9a7dc10fdf260649f4e137bf2f
---
`search article` output is verbose by default and carries a footgun sort flag, and both problems live in the same article-search surface.

Imported from March ticket 579. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/579-compact-by-default-article-search-output-with-honest-date-sort
