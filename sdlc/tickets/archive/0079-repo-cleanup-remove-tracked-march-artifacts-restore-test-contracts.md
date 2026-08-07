---
flow: build
priority: 6
---
# Repo cleanup: remove tracked .march artifacts, restore test-contracts

`make test-contracts` fails because `.march/code-log.md` is tracked in git in this branch. The repo cleanup contract (`tests/test_directory_submission_contract.py`) asserts that no `.march/`, `.claude/`, or `.agents/` paths are tracked. Additionally, `spec/13-study.md` uses `python` instead of `python3` in a bash block, which fails on systems without a `python` symlink.

Completed under March on 2026-03-29, as March ticket 079. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/079-repo-cleanup-remove-tracked-march-artifacts-restore-test-contracts
