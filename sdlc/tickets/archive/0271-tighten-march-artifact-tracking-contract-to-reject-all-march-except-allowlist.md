---
flow: build
priority: 6
---
# Tighten march artifact tracking contract to reject all .march except allowlist

`.march/verify-log.md` was accidentally committed in ticket 235 and `.march/blueprint.md` is currently tracked from ticket 246's flow, both violating `.gitignore`. The existing cleanup contract (`test_repo_cleanup_removes_local_artifacts_and_deleted_dirs_from_git`) caught them, but caught them case-by-case. The durable fix is to tighten the contract to reject ANY `.march/*` path except a small explicit allowlist, and to add a pre-commit hook that refuses to stage non-allowlisted paths.

Completed under March on 2026-04-22, as March ticket 271. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/271-tighten-march-artifact-tracking-contract-to-reject-all-march-except-allowlist
