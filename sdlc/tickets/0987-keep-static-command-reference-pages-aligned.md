---
flow: build
priority: 5
---
# Keep static command reference pages aligned

Hand-written command discovery has drifted from the shipped CLI. `list
--help` omits the working author and phenotype pages, and the human `list
discover` page omits its paging and expansion options. The adverse-event batch
wording was corrected by 0984; preserve and test that correction here rather
than replacing it.

Reconcile those pages with current behavior. The human discover reference must
state the default 5 and 1-25 bound for `--limit`, that `--offset` is a checked
zero-based index, and these exact preview/output bounds:

- Compact mode keeps at most 3 synonyms and 5 cross-references per concept,
  with at most 256 UTF-8 bytes per value and a 32 KiB structured-output budget.
- `--full` keeps at most 50 synonyms and 100 cross-references per concept,
  with at most 512 UTF-8 bytes per value and a 256 KiB structured-output
  budget.

The typed JSON pages for `list discover` and `list batch` deliberately remain
machine-readable command templates; they do not duplicate human option prose.
They must remain valid and their templates executable. Batch sections remain
described as entity-dependent on the human page, with adverse-event explicitly
unsupported.

Add an exact parity check between `catalog::entities()` and the canonical
comma-delimited names in the rendered `ListArgs.entity` help. Normalize by
lowercasing and trimming each complete comma-delimited token; aliases do not
satisfy parity. After removing the five legitimate non-entity pages
`search-all`, `discover`, `batch`, `enrich`, and `skill`, the two sets must be
equal. The test must read the production catalog and rendered Clap help rather
than restating an entity list or introducing a second command catalog.

## Done when

- Root help names every canonical list entity, including author and phenotype.
- The human discover page advertises executable options and the exact compact
  and full bounds above; the human batch page preserves 0984's truthful section
  limitation.
- A structural test fails when typed catalog entities and root help diverge.
- Human pages remain terminal-friendly; typed JSON pages remain schema-valid
  command-template responses without duplicating option prose.

## Authorized test changes

The design may add or restate assertions in `src/cli/list/tests/pages.rs`,
`src/cli/system/tests.rs`, `tests/test_json_list_contract.py`, and
`tests/list_cli_structure.rs`.
