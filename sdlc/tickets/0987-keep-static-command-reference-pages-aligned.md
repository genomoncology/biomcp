---
flow: build
priority: 5
---
# Keep static command reference pages aligned

Hand-written command discovery has drifted from the shipped CLI. `list
--help` omits the working author and phenotype pages, `list discover` omits
its paging and expansion options, and `list batch` overstates section support.

Reconcile those pages with current behavior. The discover reference must state
the default and 1-25 bound for `--limit`, the zero-based checked `--offset`,
and the bounded `--full` expansion. Batch sections must be described as
entity-dependent, with adverse-event named as unsupported.

Add a parity check between the canonical typed list catalog and the
hand-written root help so future entity additions cannot disappear silently.
Do not introduce a second command catalog.

## Done when

- Root help names every canonical list entity, including author and phenotype.
- Discover and batch pages advertise only executable options and exact bounds.
- A structural test fails when typed catalog entities and root help diverge.
- Human and JSON list pages remain parseable and terminal-friendly.

## Authorized test changes

The design may add or restate assertions in `src/cli/list/tests/pages.rs`,
`src/cli/system/tests.rs`, `tests/test_json_list_contract.py`, and
`tests/list_cli_structure.rs`.
