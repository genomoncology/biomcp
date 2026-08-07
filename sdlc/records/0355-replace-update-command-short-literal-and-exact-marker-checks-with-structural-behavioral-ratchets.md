---
base: 16b510a8da891a7ea180e59f62b844a9e5cf3d9c
head: c6d6e9d2df5e9972902e042d33736988f2e1f347
---
Issue `331-update-command-reference-ratchets.md` records three drift cases that landed in ticket 331's verify step rather than failing earlier: `src/cli/list_reference.md` advertised the old `update [--check]` after `--allow-missing-checksum` shipped; the MCP-safe description filter used an exact legacy marker that let the mutating `update` line leak into the read-only MCP tool description after the list-reference grammar changed; and the `mustmatch like "UNSAFE"` short-literal assertion was flagged by the quality ratchet, which removed the assertion in verify rather than upgrading it.

Imported from March ticket 355. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/355-replace-update-command-short-literal-and-exact-marker-checks-with-structural-behavioral-ratchets
