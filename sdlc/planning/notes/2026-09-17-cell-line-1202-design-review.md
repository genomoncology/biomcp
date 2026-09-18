# Design review: 1202 cell-line entity

Reviewed at main `e595cd15` on 2026-09-17, read-only, against the code, the 2026-09-17 ticket review (F9, F10), the dataset storage and freshness proposal (sections 7, 8, 12), workspace experiment 204, and tickets 1205, 1206, 1213, and 1214.

**Verdict: needs a fix.** Four items would send an engineer back with a question. Each is a one- or two-sentence ticket edit. No decision for Ian.

F9 and F10 are satisfied. The `section_outcomes` registry with `variants` and `xrefs` is in the design and acceptance 17, and 1205, 1206, and 1214 each add one key. The `search_idsy_molm_13_20260917.json` fixture is listed and the fixture server answers an empty window for unlisted `idsy:` queries.

## Blocking

### B1. Spec assertion 5 reads `xrefs` from a card that never fetches `dr`

The design says the card fields are `ac,id,sy,ox,di,ca,sx,ag`, "the card alone never fetches `dr`", and acceptance 7 checks that the card request has no `dr`. The executable spec then asserts `get cell-line CVCL_2119 --json` has `xrefs.depmap == ["ACH-000362"]`. Both cannot hold. Fix: make the spec line `get cell-line CVCL_2119 xrefs --json`.

### B2. The search JSON envelope has no slot for `data_as_of` or the full-window note

Acceptance 14 wants `data_as_of` and `data_as_of_kind` "at the top level of the payload" on every search output, and the design adds "one note" when the window is full. Search JSON is `SearchJsonResponseWithMeta` (`src/cli/shared.rs:568`) with exactly `pagination`, `count`, `results`, and `_meta`. `SearchJsonMeta` (`:541`) carries `next_commands`, `suggestions`, `workflow*`, `section_sources`, and `upstream_total`, and no notes. `PaginationMeta.total` is already an `Option` (`:480`), so the unknown total is covered. Fix: state where the three values live. Recommended: optional `data_as_of` and `data_as_of_kind` on `SearchJsonResponseWithMeta`, skipped when `None`, so no other entity's output changes, and `_meta.notes` as a `Vec<String>` skipped when empty.

### B3. The KG-1 fixture holds a two-species record and the design defines `species` as one value

The KG-1 probe (experiment 204, `out/cellosaurus_spelling_probes.json`) returns CVCL_1S07 (GM10888) with `species-list` `Cricetulus griseus` and `Homo sapiens`. `CellLine.species` is "taxon id and label", the search row has one `species`, and ranking says "human lines (taxon 9606) come first". Acceptance 3 renders this row. Fix: `species` is a list in upstream order, and a row counts as human when any entry is taxon 9606.

### B4. Acceptance 17's empty-variants case has no named fixture, and its last sentence names a check that does not exist

Every survey line carries 1 to 5 sequence variations, so no recorded record is truly empty. The empty case only works if the fixture server serves `/cell-line/{ac}` by path and ignores `fields`, so that `get cell-line CVCL_0007 variants` replays the U-937 file recorded without `var`. The ticket says neither. The sentence "The `spec/entity/section-outcomes.md` check accepts the new entity" points at a spec whose every block runs a drug command; nothing there enumerates entities. Fix: state that the server matches `/cell-line/{ac}` on path alone and ignores `fields`, name `get_cvcl_0007_20260917.json` as the empty-variants record, and replace the last sentence with "a `section_sources` builder for the entity in `src/render/provenance.rs`", which is where each entity's builder lives (`:205`, `:343`, `:460`, `:512`).

## Checks asked for

1. **Ranking walk.** KG1 rows from the probe: CVCL_E3VV is mouse and matches on synonym `KG1` of identifier `B16-F10.9.KG1`; CVCL_0374 is human and matches on identifier `KG-1`; CVCL_UD72 is mouse and matches on identifier `KG1`. Identifier group: 0374 (human), UD72 (mouse). Synonym group: E3VV. CVCL_0374 is first, and the ticket's reason holds. Without the human rule 0374 would still lead UD72 on upstream order, so the test does not depend on the species rule. NB4: 25 rows, two exact. CVCL_0005 matches on identifier `NB4`, CVCL_8821 (SJNB-4) on synonym only. CVCL_0005 is first, CVCL_8821 later, as acceptance 4 says.
2. **Hyphen rule.** `MOLM-13` has both fixtures. `KG-1` has both fixtures. Match kind cannot disagree between windows: it is computed after the merge from the normalized query, and `KG-1` and `KG1` normalize to the same string. What is undefined is upstream order across two windows (see N4). No test pins it.
3. **Fixture coverage.** Every acceptance item and spec assertion has a fixture or needs none, with two exceptions: acceptance 17's empty case (B4) and the `ac:` branch of acceptance 6, which is a plan test and passes, but the fixture server lists `dr:`, `idsy:`, and `id:` only (N5).
4. **Contradictions.** None with the proposal (sections 7, 8, 12 are each met), none with 1205, 1206, 1213, or 1214, and none with the survey except the NB4 fixture wording (N2). The design sentence about `is_allowed_mcp_command` is wrong about the code (N1).
5. **Out of scope.** Nothing listed is needed by an acceptance item or a dependent ticket. 1213 builds `id:` batches on the client, which is a wrapper question (N7), not a scope one.

## Non-blocking

- **N1.** `is_allowed_mcp_command` (`src/mcp/shell.rs:479`) reads `Commands::Search { .. } | Commands::Get { .. } | Commands::Author { .. } => true`. Search and get entities are not matched by name, so there is no `SearchEntity::CellLine` or `GetEntity::CellLine` arm to add. The Current Fact is right for top-level commands. The design sentence should say the raw tool admits the new entity with no change there. Acceptance 12 holds as written.
- **N2.** `search_idsy_nb4_20260917.json` is described as "the full window". The probe returned 25 rows. Say "all 25 rows" so nobody expects the 1000-row note on the NB4 spec.
- **N3.** `search_idsy_molm_13_20260917.json` returns ten rows with one exact. State the count, as the other fixture lines do.
- **N4.** Define upstream order after a merge: rows from the raw-query window in their order, then rows seen only in the hyphen-stripped window in theirs.
- **N5.** Add `ac:` to the fixture server's empty-window list, or record `search_ac_cvcl_2119_20260917.json`, so `search cell-line CVCL_2119` has a defined route.
- **N6.** State the Markdown attribution line for `data_as_of_kind: "retrieved"`, since "naming the release" in acceptance 15 has no release to name in that case.
- **N7.** Expose one `search(q, fields, rows)` under `search_names` and `search_xref`. 1213 then adds an `id:` wrapper instead of a third request path.

## Citations checked

All resolve at `e595cd15`.

| Citation | Resolves to |
|---|---|
| `src/cli/list/catalog.rs:38` | `const ENTITY_FLAGS` |
| `src/cli/list/catalog.rs:52` | `("adverse-event", true, true)` |
| `src/cli/list/catalog.rs:78` | `"pathway" => PATHWAY_SECTION_NAMES` |
| `src/cli/commands.rs:269` | `pub enum SearchEntity` |
| `src/cli/commands.rs:507` | `biomcp search adverse-event` example |
| `src/cli/commands.rs:522` | `pub enum GetEntity` |
| `src/cli/outcome.rs:129`, `:232` | `GetEntity::Pathway`, `SearchEntity::Pathway` |
| `src/cli/response_contract.rs:126`, `:287` | `GetEntity::Pathway`, `SearchEntity::Pathway` |
| `src/mcp/shell/typed_get.rs:12` | `typed_get_capabilities()` |
| `src/mcp/shell.rs:285` | eight-entity typed search list |
| `src/mcp/shell.rs:472` | `fn is_allowed_mcp_command` (see N1 for `:479`) |
| `src/mcp/shell.rs:1825` | typed rejection test for pathway |
| `src/entities/pathway.rs:18` | `default_pathway_section_outcomes()` |
| `src/sources/rate_limit.rs:167` | KEGG `policy(...)` at 334 ms |
| `src/cli/health/catalog.rs:410` | `SourceDescriptor {` |
| `src/cli/pathway/mod.rs:6`, `:28` | `PathwaySearchArgs`, `PathwayGetArgs` |
| `docs/sources/index.md:44` | KEGG row |
| `mkdocs.yml:70` | KEGG nav entry |
| `docs/reference/source-licensing.md:114` | Tier 1 heading |
| `docs/reference/sources.json:781` | `"id": "kegg"` |
| `scripts/run-specs.sh:22` | `spec/entity/pathway.md` in `SPEC_ROUTINE_PATHS` |
| `spec/fixtures/setup-provider-contract-spec-fixture.sh:474`, `:619` | KEGG path match, `BIOMCP_KEGG_BASE` export |
| `env_base`, `shared_client` | `src/sources/mod.rs:431`, `:1167` |
| 153 `real_and_receipted` receipts | confirmed by count |
| `grep -rli cellosaurus` | one Europe PMC body only, confirmed |
