---
base: fcfee10eb2b1c6c4832aadeb5dcbbf9bbc5bdf92
head: eb966edcb6dfd8f6bd7ee63f420fe212056efbaf
---
Successful JSON entity searches are supposed to teach the next executable step through `_meta.next_commands`. The review found `search protein --json` and `search phenotype --json` returning only `pagination`, `count`, and `results` because they use a bare generic search JSON helper. That is an agent-facing correctness gap: scripts get a valid JSON object but no follow-up contract.

Imported from March ticket 460. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/460-normalize-protein-and-phenotype-json-search-next-commands
