# Live checks may use Ian's keys

Date: 2026-10-21. Ian's ruling.

Ian: "you can use all my keys." And: "the keys are in the env zshenv.local don't see why this is an issue now."

BioMCP's live checks, captures and smokes may call keyed sources with the keys already set in the build machines' environment.

- A run inherits a key from the environment. No agent inspects, prints, copies, records or stores a key value. Logs and fixtures carry no key or authorization header.
- Committed fixtures stay minimized. BioMCP is public, so licensed content from OncoKB, DisGeNET, UMLS and DrugBank stays out of the repo. Raw keyed replies stay in private experiments.
- Requests stay read-only and paced within each provider's published limits.
- KEGG stays uncalled. A key does not grant a commercial license.

Reason: 0.9.4 left 5 Semantic Scholar recordings unchecked because live checks were keyless and the keyless retry hit a rate limit. No keyed path had a live check with a real key.

Replaces the keyless rule for live captures in the 2236 capture rulings. The KEGG and licensed-content rules stay.
