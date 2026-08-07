---
base: 74fe703e0b2380c1c31f5df6c63f3f4dca8f6bba
head: d31695570e616f25001ebd1faafa6dbbb6769206
---
WikiPathways API is returning HTTP 404 with a GitHub Pages "File not found" HTML page. This causes `search all --counts-only` to emit a WARN-level log line containing hundreds of characters of raw HTML into stderr, and pathway search to report "timed out" even when the real issue is a dead upstream.

Imported from March ticket 163. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/163-fix-wikipathways-404-in-pathway-search
