---
base: eaff86820a47081e4cf1c5f437c4abd09ac823e5
head: 5d5dfae907bc8fc31d399b05e3c8e7b1bf387b3f
---
The review found multiple CLI contract drifts after `make lint`, `make test`, and `make spec` were green: JSON entity searches without `_meta.next_commands`, accepted aliases hidden from help, shell-unsafe copy/paste examples, and `biomcp list` pages that omit runnable helpers. The root problem is not one command; it is that the declared CLI contract is spread across clap, `--help`, `biomcp list`, two CLI reference docs, and specs without one routine ratchet that compares them.

Imported from March ticket 459. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/459-add-whole-surface-cli-contract-ratchet-for-help-list-docs-specs-json-meta
