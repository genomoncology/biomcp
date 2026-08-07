---
base: a778c732cdcefea20fdc71ad0f77024314e376bd
head: 36a9c23bfc7505287c4f77d26a6e1d0287ff8539
---
Biomcp already has tiered validation lanes — `make check`, `make spec-pr`, full `make spec`, `make test-contracts` — but the build-flow prompts point at hardcoded `make check` / `make spec` commands and re-run them at every step transition. The path to stop that duplication is a small config file that declares which biomcp lane corresponds to each build-flow profile tier, so that once the march-side profile system is live the build flow consumes the right tier per step instead of the heaviest lane six times per ticket.

Imported from March ticket 186. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/186-declare-biomcp-validation-profiles-and-standardize-the-local-pre-commit-hook
