---
base: 810f9d8f7700174956b7e3735887a53645bf8bc9
head: 9606810c2d71de2003bcad6b32a1ba5ac591b9cc
---
Seven mustmatch assertions in the spec-v2 corpus were authored as short literals (under the 10-char ratchet) and got removed in verify because they were both syntactically below threshold and semantically too weak (e.g. `mustmatch like "suggest"` would pass on the substring "suggested"). The runtime contracts they were meant to protect are now uncovered.

Imported from March ticket 308. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/308-sweep-seven-removed-short-literal-spec-v2-assertions-with-stronger-replacements
