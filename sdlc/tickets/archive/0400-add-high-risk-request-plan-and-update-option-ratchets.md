---
flow: build
priority: 10
---
# Add high-risk request-plan and update-option ratchets

The review confirmed the highest-risk post-migration gaps are not broad runtime rewrites but missing ratchets around boundaries that already proved fragile: the update `--allow-missing-checksum` UNSAFE marker, MyDisease path/query separator rejection, and request-plan seams whose tests can pass while executors stop consuming the planned path/query/auth/cache fields. Security and correctness should be pinned by executable tests, not by planning prose.

Completed under March on 2026-06-08, as March ticket 400. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/400-add-high-risk-request-plan-and-update-option-ratchets
