---
flow: quickfix
priority: 3
---
# build.rs: re-run on git HEAD/ref changes so the baked version string stops going stale

Add cargo:rerun-if-changed on the resolved git HEAD/ref/packed-refs so build.rs re-stamps BIOMCP_BUILD_GIT_SHA when HEAD moves; must resolve the real git dir (worktree-correct, since March builds in worktrees). Low-priority build hygiene; not release-blocking.

Completed under March on 2026-06-24, as March ticket 444. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/444-build-rs-re-run-on-git-head-ref-changes-so-the-baked-version-string-stops-going-stale
