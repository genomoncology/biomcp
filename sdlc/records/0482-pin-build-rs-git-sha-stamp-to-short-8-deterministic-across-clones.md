---
base: d41c239970e39a3792495b9c1cedcb157d6561c2
head: 0257a91c407916a1808af730b9df99598a0efb16
---
The `Release` workflow's `validate` job now gets past `make spec` but fails in `scripts/release-smoke.sh`, which asserts the release binary's stamped git SHA matches `HEAD`. The binary is stamped with an **adaptive-length** short SHA while release-smoke compares against a **fixed 8-char** short SHA, so they diverge by environment:

Imported from March ticket 482. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/482-pin-build-rs-git-sha-stamp-to-short-8-deterministic-across-clones
