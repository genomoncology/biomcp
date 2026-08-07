---
base: ac00defbddf335df3f9e15f0159271c152af8ea1
head: eda3d9cbc421381dde4b6f5880e08b2a0e89862c
---
BioMCP's suggested follow-up commands ("See also", "More") are largely template-based and don't use what was actually found in the current response to generate targeted suggestions. For example, when `get disease "Dravet syndrome"` returns SCN1A as the causal gene with score 0.872, the suggestions should include `biomcp get gene SCN1A clingen constraint` — not generic templates like `biomcp search article -d "Dravet syndrome"`.

Imported from March ticket 162. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/162-improve-suggested-next-commands-with-context-aware-routing
