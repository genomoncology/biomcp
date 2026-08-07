---
base: 0a631ddbd1ca82a13c9b615a4be6b8cc7cdaa5ed
head: 8e2101445ef78d3ef701f68473c562961fc5df97
---
Ticket 342 fixed CTGov intervention alias preservation in trial next-commands and verify repaired shell escaping for source-derived next-command arguments at runtime. Issue `342-ctgov-alias-next-command-spec-ratchets.md` records two remaining ratchet gaps that should be pinned at design rather than patched in verify:

Imported from March ticket 357. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/357-add-behavioral-ctgov-intervention-alias-and-shell-safety-ratchets-to-trial-spec
