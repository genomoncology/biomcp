---
flow: build
priority: 3
hold: the replacement generator is not chosen yet; Ian picks before promotion
---
# Move the documentation build off unmaintained MkDocs

The `make test` gate ends in `mkdocs build --strict`, and that build now prints a warning on every run: MkDocs 1.x is unmaintained, and MkDocs 2.0 is incompatible with Material for MkDocs. The Material authors are directing users to Zensical, their own replacement generator. Nothing is broken today — the build passes and the published site is correct — but the toolchain under it has no maintainer, so the next security advisory or Python version bump lands with no upstream fix available.

Docs are the public face of BioMCP. `biomcp.org` is served from the `gh-pages` branch, and the site must keep serving the same pages at the same URLs throughout and after this change. A migration that renames or drops URLs breaks inbound links from the MCP registry, the README, and anything an agent has already learned.

The current setup is small, which is what makes this tractable: one built-in plugin (`search`), the Material theme with a features list, a hand-written `nav`, and eleven Markdown extensions, all in `mkdocs.yml`. There are no custom plugins and no generated navigation.

## The decision that comes first

This ticket must not run until someone chooses the destination. The realistic options are to move to Zensical, to stay on MkDocs 1.x behind an explicit pin and accept it as unmaintained, or to move to a different generator entirely. Those lead to genuinely different work, and an agent cannot pick for us. When the choice is made, write it into this ticket as a single named destination and delete the `hold:` line.

## Done when

- The documentation build runs on a maintained toolchain, with no unmaintained-dependency warning in the output of `make test`.
- Every page reachable in the published site before the change is reachable at the same URL after it.
- The rendered output still carries the things the current theme provides and the docs rely on: working search, admonitions, tabbed content blocks, syntax-highlighted code with line anchors, snippet includes, and permalinked headings.
- The strict build still fails on a broken internal link, the same way `--strict` does today.
- The build still runs inside the offline sandbox with no public network access, exactly as `make test` invokes it now.
- The publish path to `gh-pages` still works and is exercised at least once before this ticket is called done.

## Existing tests that pin this

These assert against the docs tree or the docs build and may need restating as part of the design stage. Restatement is authorized for these files by name:

- `tests/test_docs_changelog_refresh.py`
- `tests/test_source_pages_docs_contract.py`
- `tests/test_source_licensing_docs_contract.py`
- `tests/test_public_search_all_docs_contract.py`
- `tests/test_cross_entity_pivots_docs_contract.py`
- `tests/test_bioasq_benchmark_contract.py`
- `tests/test_pre_commit_reject_march_artifacts.py`
- `tests/surface/test_ticket_405_architecture_operator_contracts.py`

The dependency pin lives at `pyproject.toml:35` (`mkdocs-material>=9.5`); the build command is `Makefile:41`.
