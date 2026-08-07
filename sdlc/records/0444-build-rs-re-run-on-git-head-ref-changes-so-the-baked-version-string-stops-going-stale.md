---
base: fc2922fd8e970f1f7dacaa68f2868a13709cc58d
head: ec0062ab1ae360ab1be949cc85c570fdf58cdf82
---
Add cargo:rerun-if-changed on the resolved git HEAD/ref/packed-refs so build.rs re-stamps BIOMCP_BUILD_GIT_SHA when HEAD moves; must resolve the real git dir (worktree-correct, since March builds in worktrees). Low-priority build hygiene; not release-blocking.

Imported from March ticket 444. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/444-build-rs-re-run-on-git-head-ref-changes-so-the-baked-version-string-stops-going-stale
