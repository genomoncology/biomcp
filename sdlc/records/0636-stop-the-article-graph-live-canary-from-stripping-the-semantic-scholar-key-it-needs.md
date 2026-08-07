---
base: b6dcfc7199e0f208f21ecbc332a2d27d6b194ba3
head: 510c49982b414ddb617505ce5e52301d0d3a345d
---
article-graph-live.md runs through tools/biomcp-ci, which unsets S2_API_KEY, so a credentialed verify page hits the anonymous Semantic Scholar rate limit and fails.

Imported from March ticket 636. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/636-stop-the-article-graph-live-canary-from-stripping-the-semantic-scholar-key-it-needs
