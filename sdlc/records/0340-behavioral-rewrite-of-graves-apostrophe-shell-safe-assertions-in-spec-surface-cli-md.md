---
base: 9d82f5132cd997c5be5c8aba554c713c04bfc6a7
head: e5eb17ef64066537495cad117008013431d6759a
---
Three `mustmatch like "Graves'"` assertions in `spec/surface/cli.md` were converted to regex form during ticket 313 verify because the quality ratchet enforces a 10-character minimum on `like` literals. The regex form tests the same thing but leaves a behavioral coverage gap: the assertion does not exercise the surrounding command structure (arg name, quoted form, full argv shell-safety). The 327 review classified this as a post-release behavioral rewrite — small enough to ship as a quickfix.

Imported from March ticket 340. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/340-behavioral-rewrite-of-graves-apostrophe-shell-safe-assertions-in-spec-surface-cli-md
