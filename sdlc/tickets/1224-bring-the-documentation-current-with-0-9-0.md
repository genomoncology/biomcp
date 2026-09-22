---
flow: build
priority: 3
deps: [1222]
---

# 1224: Bring the documentation current with 0.9.0 and close the site drift gaps

## Goal

The published site matches 0.9.0's shipped surface and stops drifting from it. A reader who follows the docs gets commands, values, and facts the released binary accepts.

## Current Facts

Audited 2026-09-22 at main `b17143b7` against the released image `ghcr.io/genomoncology/biomcp:0.9.0` and the live site. The site is current with main (gh-pages tip `bc8b9f02` deployed `b17143b7`; the live revision witness returns 200 and the Markdown bytes match), but main's content misses 0.9.0 surfaces and carries wrong or unguarded facts.

Shipped but undocumented:

- The 0.9.0 breaking refusal of bare `--status active` appears in zero docs pages. `search trial --help` lists `active_not_recruiting` and `enrolling_by_invitation`, but the status table omits `enrolling by invitation` and never states the refusal (`docs/reference/quick-reference.md:222-236`); the trial guide lists NCI mappings only (`docs/user-guide/trial.md:120-126`). The comma form `active, not recruiting` is still accepted as an alias.
- The trial zero-result hint, the relaxation suggestion, and `_meta.upstream_total` are undocumented. The 0.9.0 changelog claims the trial guide was refreshed, but `git show v0.9.0:docs/user-guide/trial.md` has no hyphen or zero-result content.
- The `author` entity (new in 0.9.0) is absent from the entity tables in `README.md`, `docs/index.md:125-140`, and `docs/reference/quick-reference.md`; only `docs/user-guide/author.md` and `docs/user-guide/cli-reference.md:1026-1031` cover it.
- `study top-mutated` has no command entry (`docs/reference/quick-reference.md:153-161`, `docs/user-guide/cli-reference.md:995-1009`).
- The provider-capture store (`provider_capture_bytes`, `provider_capture_bytes_freed`) is absent from `docs/`, while `docs/policies.md:36-47` inventories the managed trees and cannot explain the fourth one.
- `biomcp mcp tools` is documented nowhere.

Wrong or stale:

- `docs/reference/quick-reference.md:235` lists `unknown status` as a valid `--status` value; the binary rejects it.
- `docs/getting-started/installation.md:22` pins the install example to `--version 0.8.0`.
- `docs/reference/mcp-server.md:17` still says v0.8.25. Ticket 1222 owns the fix; 1224 only verifies it.
- `docs/reference/mcp-server.md:146-148` and `docs/getting-started/claude-desktop.md:47-50` carry hand-copied catalog numbers (15,841 bytes, 3,996 tokens) pinned as literals by `tests/test_documentation_consistency_audit_contract.py:492-494`, with nothing measuring them against the binary.
- `docs/reference/sources.json` `reviewed_on` dates are format-checked but not age-checked (`tests/test_source_licensing_docs_contract.py:182`); many read `2026-03-20`.
- `docs/reference/release-process.md:87-89` version facts have no mechanical check; the same facts in `architecture/technical/overview.md:268` are ticket 1222's.
- The cell-line entity merged to main after v0.9.0 (`eed4f2c1`) has no Unreleased CHANGELOG entry.

Site structure:

- `docs/conftest.py` is published (`https://biomcp.org/conftest.py` returns 200); `mkdocs.yml` sets no `exclude_docs`.
- No changelog or release-notes page is in the nav; `CHANGELOG.md` lives outside `docs/`.
- The `llms.txt` curated tool list's composition and the `llms-full.txt` one-line descriptions' content are unguarded; page coverage and the seven tool names are checked (`tests/test_llms_txt_contract.py:79-108`).

Enforcement gap:

- The CLI surface ratchet (`tests/test_cli_surface_contract_ratchet.py`, `tools/check-quality-ratchet.py:1017-1318`) reads every docs page but asserts nothing about guide content beyond clap-alias and command tokens. It never checks JSON field names, value vocabularies, version strings, `README.md`, or the `llms` files, which is why the items above pass a green ratchet.

## Design

- Fix the shipped-but-undocumented items in the owning pages: trial guide and quick reference (status values, the bare `active` refusal, zero-result hint, `upstream_total`, hyphenated-term behavior); entity tables in `README.md` and `docs/index.md`; quick reference and CLI reference (`author`, `study top-mutated`, `mcp tools`); `docs/policies.md` for the provider-capture store.
- Correct the wrong values: drop `unknown status`, update the install example, move the version facts to v0.9.0, and replace the hand-copied catalog numbers with a measured value or remove the exact numbers.
- Add `exclude_docs` for `docs/conftest.py`, and publish `CHANGELOG.md` as a docs page in the nav with its described entry in `docs/llms-full.txt`.
- Extend the ratchet with the cheapest guards that bite: the trial status vocabulary, the `author` entity in the entity tables, and the release-process version facts compared against `Cargo.toml` and `pyproject.toml` rather than hard-coded. Keep each guard scoped so it fails on a wrong sentence, not on reflow.
- Refresh the `sources.json` review dates or add an age check, and add the Unreleased CHANGELOG entry for the cell-line entity.

## Acceptance

1. Every fact named in the findings is documented correctly or removed from the docs.
2. `docs/reference/quick-reference.md` and the trial guide agree with `biomcp search trial --help` and the bare `--status active` refusal, and keep the `active, not recruiting` alias.
3. The ratchet carries at least the status vocabulary, the `author` entity, and the release-process version facts, with the versions compared to `Cargo.toml` and `pyproject.toml`.
4. `https://biomcp.org/conftest.py` is gone after the next deploy, the changelog page is in the nav and in `llms-full.txt`, and `mkdocs build --strict` passes.
5. The Unreleased CHANGELOG names the cell-line entity, and every `reviewed_on` date is refreshed or an age check rejects a stale one.
6. `make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- The release and docs coupling, which is ticket 1222.
- Cell-line pages for surfaces merged after v0.9.0; they ship in the next release.
- The MCP conformance findings in ticket 1223.

## Complexity

- Contract score: 1 (several explicit public cases: documented values, entities, and version facts)
- State and timing score: 0 (documentation and guards only)
- Reach score: 1 (one public surface, the published site, plus the ratchet)
- Proof score: 1 (focused doc and ratchet checks against the released binary)
- Cost of error score: 1 (a reader follows a wrong value or misses a breaking change)
- Total: 4
- Minimum level floor: none
- Final level: 2
- Reasons: many small public-content corrections with deterministic doc and ratchet proof; no state, concurrency, or external effect
- Selected model: gpt-5.6-luna, high reasoning (level 2 implementer)

## Review

- Design review: ACCEPT 2026-09-22 (gpt-5.6-sol, medium) — one round; three citation/wording defects and the acceptance-coverage gap corrected in this ticket, no split needed
- Code review: pending
