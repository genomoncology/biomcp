---
flow: build
priority: 5
---
# Replace update-command short-literal and exact-marker checks with structural behavioral ratchets

Issue `331-update-command-reference-ratchets.md` records three drift cases that landed in ticket 331's verify step rather than failing earlier: `src/cli/list_reference.md` advertised the old `update [--check]` after `--allow-missing-checksum` shipped; the MCP-safe description filter used an exact legacy marker that let the mutating `update` line leak into the read-only MCP tool description after the list-reference grammar changed; and the `mustmatch like "UNSAFE"` short-literal assertion was flagged by the quality ratchet, which removed the assertion in verify rather than upgrading it.

Completed under March on 2026-04-29, as March ticket 355. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/355-replace-update-command-short-literal-and-exact-marker-checks-with-structural-behavioral-ratchets
