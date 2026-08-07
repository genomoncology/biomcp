---
base: 7af8774e0acf88037473c7d975e18112666ec861
head: 241d81f8e53521a6dea0093fc0e3024f3dfe1fd6
---
70+ commits have landed since v0.8.19 (tickets 075-087) covering product fixes, cross-entity links, quality ratchet, architecture docs, and repo cleanup. The quality ratchet script (`tools/check-quality-ratchet.sh`) inlines mustmatch lint logic because `mustmatch lint` wasn't on PyPI until 0.0.4, which was just published. This release removes that shim, updates the changelog, and tags.

Imported from March ticket 088. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/088-v0-8-20-release-changelog-mustmatch-shim-removal-tag
