---
base: 43ddb0fe828d24b86dda683d4635bf440cbbf2be
head: 183d1ad2efc457d615b0f97d99a115659a971165
---
The review confirmed the highest-risk post-migration gaps are not broad runtime rewrites but missing ratchets around boundaries that already proved fragile: the update `--allow-missing-checksum` UNSAFE marker, MyDisease path/query separator rejection, and request-plan seams whose tests can pass while executors stop consuming the planned path/query/auth/cache fields. Security and correctness should be pinned by executable tests, not by planning prose.

Imported from March ticket 400. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/400-add-high-risk-request-plan-and-update-option-ratchets
