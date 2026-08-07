---
base: 267e701cbc965980fb598c96e172f913693895ff
head: 843e7a311dd65f4a13a8249a7cae9ddd438350b2
---
The 700-line cap on `src/cli/**/*.rs` is a durable architecture rule in `architecture/technical/cli-module-decomposition.md`, but only the recently decomposed areas have structure ratchets (search_all, health, suggest, skill, list, article tests, benchmark). `make check` can pass while new or out-of-scope files exceed the cap. The 327 review found six current over-cap files that no ratchet covers:

Imported from March ticket 334. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/334-add-global-src-cli-line-cap-ratchet-with-allowlist
