---
flow: build
priority: 7
---
# Architecture docs: See-also contract, discover grammar, CI description, post-tag proof

The holistic review (076) found that the See-also / next_commands rendering system has no architectural home — two specs protect it, the code review found it broken, but the architecture docs contain zero mention of this feature. Additionally, the `discover` command is shipped and spec'd but missing from the command grammar in both `functional/overview.md` and `ux/cli-reference.md`. The CI `check` job description falsely claims the quality ratchet runs in CI. The post-tag proof section is hardcoded to v0.8.18 while the repo is at 0.8.19.

Completed under March on 2026-03-29, as March ticket 078. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/078-architecture-docs-see-also-contract-discover-grammar-ci-description-post-tag-proof
