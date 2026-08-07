---
flow: build
priority: 6
---
# Push duplicated spec proof down to unit tests and declare hidden fixture dependencies

Move spec assertions that duplicate existing unit tests down to the unit layer, stop specs from invoking cargo, and declare the undeclared CTGov fixture dependency

Completed under March on 2026-07-29, as March ticket 624. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/624-push-duplicated-spec-proof-down-to-unit-tests-and-declare-hidden-fixture-dependencies
