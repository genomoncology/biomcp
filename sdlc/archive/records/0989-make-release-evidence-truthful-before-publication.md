---
base: e6e6892d
head: b6084b6a
---

Release inspection is now the single fail-closed writer of assembled-artifact
evidence. It verifies artifact bytes, platform evidence, SBOMs, signing data,
wheel identity and RECORD contents, and exact structured binary identity rather
than retaining caller claims. All five private wheel jobs install and smoke both
commands before sealing or publication.

Promotion binds normalized manual inputs, the updater result, and the actual
latest public release into one inventory consumed by publication and
reconciliation. Curated release notes come from the matching changelog block.
Tampered evidence, malformed wheels, stale public-release claims, and missing or
incorrect identity leave no successful record. Focused release suites, canonical
lint, independent review, and the later exact-head hosted gates passed.
