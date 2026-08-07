---
flow: build
priority: 9
---
# Add whole-surface CLI contract ratchet for help list docs specs JSON meta

The review found multiple CLI contract drifts after `make lint`, `make test`, and `make spec` were green: JSON entity searches without `_meta.next_commands`, accepted aliases hidden from help, shell-unsafe copy/paste examples, and `biomcp list` pages that omit runnable helpers. The root problem is not one command; it is that the declared CLI contract is spread across clap, `--help`, `biomcp list`, two CLI reference docs, and specs without one routine ratchet that compares them.

Completed under March on 2026-06-29, as March ticket 459. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/459-add-whole-surface-cli-contract-ratchet-for-help-list-docs-specs-json-meta
