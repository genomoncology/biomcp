---
flow: build
priority: 5
---

# Make the MCP `get` schema advertise each entity's actually callable sections

Status: implemented; code review pending

## Outcome

An agent can read the typed MCP `get` definition and know the exact ordinary
section tokens that are callable for the selected entity. The schema and typed
request mapper use one MCP capability projection, so their entity, section, and
section-cardinality behavior cannot drift. Independent tests anchor that
projection to the CLI inventory and the explicit MCP binary-download policy
rather than using the projection itself as the expected result.

## Current facts

The 2026-09-01 measured run remains useful evidence, but the original account
of its cause was too broad. BioMCP `0.8.25` served four MCP tools over stdio via
`biomcp serve`. Claude Code `2.1.252` in headless mode drove them across 31
biomedical tasks, one fresh process per task, model `claude-sonnet-5`, with no
other tools available. Of 179 BioMCP tool calls, 52 failed (29.1%), and 19 of
those 52 used a section token on an entity that did not accept it. The observed
tokens were `guidelines`, `guidance`, `conditions`, and `ontology`.

No raw transcript or aggregate from that run is checked into this repository;
the counts above are retained as the ticket's dated field evidence rather than
presented as independently reproducible from the checkout. The code-level
cause is independently verifiable at the tagged release and current main.

Those tokens were not nonexistent, as the first version of this ticket said.
They are real sections of different entities: the `v0.8.25` entity constants
assign `guidelines` to PGx, `guidance` to adverse events, `conditions` to
diagnostics, and `ontology` to genes. The released `v0.8.25` implementation in
`src/mcp/shell.rs` did advertise section names, but exposed one global union
from `all_get_sections()` for every typed `get`. It therefore told the agent
that these cross-entity combinations were legal and rejected them only after a
call. The measured run exercised the released July `v0.8.25` line, not the
newer work already present on main.

Commit `3231f49b` (`0933: make typed MCP schemas entity-specific`, 2026-08-12)
already fixed that principal defect for the unreleased 0.9 line. On current
`0.9.0-dev.6`, `src/mcp/shell/typed_get.rs` emits one branch per entity and
reads each enum from `crate::cli::list::catalog::sections(entity)`; `get_args`
uses the same catalog function when validating ordinary section tokens.
Current `biomcp mcp tools` output has 12 entity-specific branches, omits
`sections` for `author`, and advertises the four formerly misapplied tokens only
on their owning entities. The existing Rust
`typed_schemas_are_entity_specific` test and the stdio unknown-section contract
pass.

One observable mismatch remains. The current article schema advertises:

```text
annotations, fulltext, tldr, indexing, assets, asset, all
```

`asset` is a binary-download form requiring a following filename. Typed MCP
cannot call it: `get_args` rejects an article request beginning with `asset`
with `Binary article asset downloads are CLI-only`, and the common MCP
execution boundary rejects the corresponding raw command. The safe JSON
manifest selector `assets` remains callable. Thus the current schema still
names one section that its tool refuses, and an agent reading only the schema
cannot distinguish it from `assets`.

Trial has the analogous variadic terminal forms `documents` and `document
<filename>`, but neither is in `TRIAL_SECTION_NAMES`, so neither is currently
advertised or accepted by typed `get`. Raw MCP rejects `document <filename>` at
the common binary boundary. This ticket does not broaden typed MCP to either
trial terminal form; it must preserve that behavior while repairing the one
actual schema/mapper mismatch, article `asset`.

The present proof is not exact enough to catch this. The unit test checks only
that gene contains `pathways` and not `population`; the HTTP executable
contract checks representative `pathways` and `indexing` values plus a literal
12-branch count. Neither compares every advertised enum with the typed mapper's
callable set. The mapper and schema also repeat their 12-entity list separately,
so a future gettable entity can drift even though ordinary section validation
shares the CLI catalog.

The current real catalog is comfortably inside the existing budget:

```text
tools/list UTF-8 bytes: 16000 (ceiling: 22600)
tools/list cl100k_base tokens: 4066 (ceiling: 5800)
biomcp description UTF-8 bytes: 211 (ceiling: 4000)
```

## Test-first design

1. Add a failing generic Rust agreement test before changing the schema. Build
   its expected entity inventory directly from the gettable rows of
   `cli::list::catalog::entities()`, not from the new MCP projection. Build each
   expected ordinary-section set directly from the CLI catalog minus a
   test-owned assertion that article `asset` is the sole catalogued CLI-only
   binary form. Do not call the production projection to compute expected
   branches or enums. Require exactly one schema branch per expected entity and
   no extras; require `author` to omit `sections`; and require every other enum
   to equal its independently derived expected set. For each expected section,
   construct a minimal `TypedGet` and prove `get_args` maps it without provider
   work. Also assert that the CLI catalog still contains article `asset`, then
   call `get_args` with the real variadic form `asset, fixture.bin` and require
   the CLI-only rejection. This test must fail on today's advertised `asset`.
2. Add a separate inventory alignment test between the CLI catalog's gettable
   names and the actual Clap `get` subcommands (the same `CommandFactory`
   introspection formerly used by the MCP code is sufficient). This keeps the
   typed CLI catalog as the projection's deliberate owner without allowing a
   newly added `GetEntity` variant to disappear merely because both MCP schema
   and mapper read a stale `ENTITY_FLAGS` table.
3. Introduce one typed-MCP get capability projection owned beside
   `typed_get_schema`/`get_args`. Derive its entity inventory and ordinary
   section names from `cli::list::catalog`; encode `author` as having no section
   input, preserve adverse-event's idempotent duplicate behavior, and apply the
   explicit MCP trust-policy exclusion for the CLI-only article `asset`
   variadic download. Both schema generation and mapper validation, including
   `uniqueItems`/duplicate handling, must consume this same projection. Do not
   copy all entity section arrays into MCP code, and do not remove `asset` from
   the CLI/entity catalog, where it remains a valid terminal command.
4. Add focused negative and positive unit coverage for the trust boundary:
   article `asset` is absent from the typed schema and `asset, fixture.bin` is
   rejected by `get_args` before file or provider access, while article
   `assets` remains advertised and maps to the existing JSON-safe manifest
   command. Preserve regression coverage for both raw binary forms, article
   `asset <filename>` and trial `document <filename>`, including their CLI-only
   guidance. Assert that trial `document` and `documents` remain absent from
   typed sections so this narrowing cannot accidentally broaden the surface.
5. Strengthen the real transport/catalog contract rather than relying only on
   an in-memory schema. Extend the existing `typed-tools` HTTP executable
   contract to prove entity-specific ownership of the four measured tokens and
   to prove the article branch advertises `assets` but not `asset`. Replace the
   brittle literal branch-count assertion with agreement against the owned
   gettable inventory where that comparison belongs in Rust. In the transport
   example, locate a branch by its `properties.entity.const` and inspect that
   branch's exact `sections.items.enum`; do not use the existing recursive
   `json_property_contains`, which cannot prove token ownership. The rmcp HTTP
   client's decoded `list_tools` result is the serialized `tools/list` catalog
   an agent receives and is a feasible boundary for this contract.
6. Run `scripts/measure-mcp-tools.py` against the rebuilt worktree binary and
   retain the existing byte/token ceilings unchanged. This change removes one
   enum value and must not widen the catalog budget.

## Scope

In scope: the typed `get` capability projection, entity-specific schema enums,
typed mapper validation, the article `asset` MCP exclusion, exact generic
agreement tests, serialized HTTP `tools/list` proof, and the existing catalog
budget gate.

Out of scope: changing any biomedical section name or behavior; removing the
CLI article asset download; exposing binary bytes or server-local paths over
MCP; changing the JSON-safe article `assets` manifest; changing raw MCP's
binary-download refusal text; changing search schemas; raising context-budget
ceilings; backporting to the released 0.8 line; or rerunning the 31-task agent
study.

This is a schema tightening, not a valid-call break: current typed clients
cannot successfully call article `asset`, so removing that advertised value
only makes discovery truthful. CLI users retain the command, and raw MCP callers
retain the existing explicit refusal. The implementation must continue to
reject unadvertised input server-side because clients are not required to
validate JSON Schema themselves.

## Acceptance

- The CLI catalog's gettable inventory equals the actual Clap `get` subcommand
  inventory. Serialized `tools/list` exposes one distinct branch for every
  such entity and no extra branch. `author` has no section input.
- Each section-bearing branch's enum exactly equals the set accepted by the
  typed mapper for that entity. The generic test derives its expected set from
  the CLI catalog plus its own explicit article-asset trust assertion, not from
  the production MCP projection; ordinary catalog changes flow through both
  production consumers and remain exhaustively exercised, while a projection
  or exclusion drift fails the test.
- The measured cross-entity guesses are impossible from the schema: `ontology`
  appears only for gene, `conditions` only for diagnostic, `guidelines` only
  for PGx, and `guidance` only for adverse-event.
- Article `assets` is advertised and accepted as the existing JSON-safe
  manifest selector. Article `asset` is not advertised and remains rejected by
  both typed and raw MCP before filesystem or provider work, with the existing
  CLI-only guidance. The CLI asset-download command remains available. Trial
  `document`/`documents` remain absent from typed sections, and raw trial
  `document <filename>` retains the same binary refusal.
- The HTTP MCP surface contract inspects these properties in the real
  serialized catalog an agent receives.
- `uv run --no-sync python scripts/measure-mcp-tools.py` passes without changing
  its 22,600-byte, 5,800-token, or 4,000-description-byte ceilings.
- Focused Rust schema/mapper tests, `tests/rmcp_client_contract.rs`, the affected
  `spec/surface/mcp.md` contract, `make lint`, `make test`, and `make spec` pass.
  No AlphaGenome behavior changes, so `make full-feature-check` is not required.

## Dependencies

None. Commit `3231f49b` and the installed-binary catalog command from ticket
1030 are already on main and are implementation inputs, not blockers. The
existing MCP binary-download trust boundary must be preserved.

## Implementation evidence

Implemented on 2026-09-04. `typed_get_capabilities` now derives every gettable
entity and ordinary section from the CLI catalog, represents author's absent
section input explicitly, carries the adverse-event duplicate policy, and
removes only article `asset` at the typed-MCP trust boundary. Schema generation
and request mapping both consume that projection. Independent tests derive
their oracle from the CLI catalog, compare its gettable inventory with Clap,
exercise every advertised section through the mapper, and retain positive
`assets` plus negative article/trial binary coverage. The HTTP contract now
inspects exact entity branches in the decoded serialized `tools/list` result.

The test-first oracle failed before the product change because article
advertised both `assets` and `asset`, then passed after the projection landed.
Focused verification passed:

- `cargo test --no-default-features typed_get_tests -- --nocapture` (4 passed)
- `cargo test --locked --no-default-features
  binary_downloads_are_rejected_but_manifests_remain_allowed -- --nocapture`
  (1 passed)
- `cargo test --locked --no-default-features --test rmcp_client_contract`
  (12 passed, 2 intentionally ignored live tests)
- `make spec-contracts` (99 passed/3 skipped, 38 passed/1 skipped, 6 passed,
  and 10 passed across its focused groups)
- `uv run --no-sync python scripts/measure-mcp-tools.py` after rebuilding the
  worktree binary (15,992 bytes, 4,064 tokens, 211 description bytes; ceilings
  remain 22,600/5,800/4,000)
- `cargo fmt --all -- --check`, locked no-default-features Clippy for all
  targets with warnings denied, and `git diff --check`

No material design assumption was disproved. One execution correction was
required: the repository intentionally rejects a direct `run-specs.sh`
specification invocation without Make-declared routine features, so the
supported focused `make spec-contracts` target was used.

The primary `make lint` gate subsequently found only that the implementation's
deduplication had shortened the grandfathered `src/mcp/shell.rs` from its exact
pinned 2,136-line source-size baseline to 2,124 lines. The baseline was not
changed and no padding was added. Remediation restored the exact 2,136 lines
with a focused API contract comment on `get_args` documenting why schema and
mapping share the capability projection, why article `asset` is rejected
before ordinary validation while `assets` remains safe, why trial terminal
forms stay excluded, and why adverse-event duplicates remain idempotent.
After remediation, `make lint` passed in full, including the source-size
ratchet, Clippy, license, and advisory checks. The typed-get group remained 4/4
green, the focused binary-boundary test remained green, formatting and diff
checks passed, and `wc -l src/mcp/shell.rs` reported exactly 2,136.

## Review

- Design review (2026-09-04): REJECT. The amended draft correctly identifies
  the narrow residual, historical cause, transport surface, and unchanged
  budget. It was not implementation-ready because its proposed agreement test
  used the same production projection as both implementation and oracle, its
  entity inventory was not checked against the actual `get` grammar, and it
  did not make author/no-sections, adverse-event duplicate semantics, or the
  analogous trial binary form explicit. The corrections above add independent
  oracles and exact boundary assertions; re-review is required before
  implementation.
- Design re-review (2026-09-04): ACCEPT. The corrected design is
  implementation-ready. Its generic oracle derives gettable entities from the
  CLI catalog, independently checks that inventory against the actual Clap
  `get` subcommands, and derives ordinary sections from the catalog with a
  test-owned assertion for the sole catalogued MCP exclusion rather than from
  the production projection. It explicitly preserves author's no-sections
  shape and adverse-event's idempotent duplicate semantics; covers the article
  and trial variadic binary trust boundaries without broadening trial's typed
  surface; and requires branch-local inspection of the decoded HTTP
  `tools/list` schema, eliminating the recursive cross-branch false positive.
  The compatibility claim is limited to successfully callable behavior, and
  the measured 16,000-byte/4,066-token/211-description-byte baseline was
  reproduced beneath the unchanged 22,600/5,800/4,000 ceilings.
- Code review: ACCEPT (2026-09-04). The reviewer verified that schema and mapper
  share one production projection while the tests retain independent CLI
  catalog, Clap inventory, and explicit trust-policy oracles. Author,
  adverse-event duplicates, safe article `assets`, rejected article `asset`,
  excluded trial terminal forms, raw binary guidance, and branch-local HTTP
  ownership assertions all remain correct. The source-size remediation was
  re-reviewed separately: its `get_args` contract documentation is substantive,
  no ratchet baseline changed, and no behavior changed. No blocking or optional
  findings remain.

## Completed 2026-09-04

The typed MCP `get` schema now advertises exactly the ordinary sections its
mapper accepts for each entity. The uncallable article `asset` binary selector
is absent while the safe `assets` manifest remains available, and exhaustive
agreement tests prevent entity, section, cardinality, and trust-policy drift.

Final primary gates passed on the independently accepted tree: `make lint`;
`make test`, including the complete offline Rust lane, 883 Python tests passed
and 3 skipped, and strict documentation; and `make spec`, including all routine
mustmatch pages, 38 isolation contracts, fixture cleanup, and the 8-case static
lane. The rebuilt catalog remains below unchanged ceilings at 15,992 bytes,
4,064 tokens, and 211 description bytes.
