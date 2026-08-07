---
flow: quickfix
priority: 3
---
# Behavioral rewrite of Graves apostrophe shell-safe assertions in spec/surface/cli.md

Three `mustmatch like "Graves'"` assertions in `spec/surface/cli.md` were converted to regex form during ticket 313 verify because the quality ratchet enforces a 10-character minimum on `like` literals. The regex form tests the same thing but leaves a behavioral coverage gap: the assertion does not exercise the surrounding command structure (arg name, quoted form, full argv shell-safety). The 327 review classified this as a post-release behavioral rewrite — small enough to ship as a quickfix.

Completed under March on 2026-04-29, as March ticket 340. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/340-behavioral-rewrite-of-graves-apostrophe-shell-safe-assertions-in-spec-surface-cli-md
