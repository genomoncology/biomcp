---
base: 874fe02487d65669951d20fbcc15f519638faa06
head: 8bb4a0deac6ecb80ac7232932e92ebc80cbaff6e
---
The holistic review (076) found that the See-also / next_commands rendering system has no architectural home — two specs protect it, the code review found it broken, but the architecture docs contain zero mention of this feature. Additionally, the `discover` command is shipped and spec'd but missing from the command grammar in both `functional/overview.md` and `ux/cli-reference.md`. The CI `check` job description falsely claims the quality ratchet runs in CI. The post-tag proof section is hardcoded to v0.8.18 while the repo is at 0.8.19.

Imported from March ticket 078. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/078-architecture-docs-see-also-contract-discover-grammar-ci-description-post-tag-proof
