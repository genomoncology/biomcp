---
base: 7725c4ffab1c1c2a470d6601aa755316ab3c7bd3
head: 765f7264b64d84fc21db2ff8a535e4f6f72480c0
---
biomcp test/check runs leak named scratch dirs in /tmp; add EXIT trap cleanup

Imported from March ticket 248. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/248-trap-clean-test-scratch-dirs-in-tmp
