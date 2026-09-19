# Review of tickets 1202 to 1216: cell line, dataset, import, compare, score

**Status 2026-09-19: the cell line rows are settled. Tickets 1202, 1205, 1213
and 1214 took their fixes, shipped, and gated green on the build host; each has
a completion record in `sdlc/records/`. Ticket 1206 is deferred and held as a
draft over the DepMap terms contradiction, so its findings F5, F11, F12, F24 and
F25 are parked with it. The dataset rows below are untouched and still hold.**

Written 2026-09-17 against main at `45d74711`. Reviewed: the fifteen tickets, the proposal `sdlc/planning/2026-09-17-dataset-storage-and-freshness.md`, the findings of workspace experiments 203 (GEO contract proof) and 204 (cell line source survey), and the code the tickets cite. Two read-only public requests filled gaps the experiments left: one GEO matrix header (GSE100446) and one `gds` ESummary record (GSE982). Nothing was edited in the tickets.

Verdict scale: **ready** means an engineer can build it as written and the findings below are polish. **Needs a fix** means the ticket text must change before it is dispatched. **Needs a decision** means the lead or Ian has to choose.

## Verdicts

| Ticket | Verdict | Why |
| --- | --- | --- |
| 1202 cell-line entity | needs a fix, fixed and shipped | Skipped `section_outcomes`. 1205 and 1214 then needed it (F9). Missing one fixture the hyphen rule requires (F10). Both applied; shipped 2026-09-18. |
| 1203 search dataset | ready | Polish only (F7, F19). |
| 1204 dataset card | needs a fix | The `Public` comparison is wrong (F1). `data_as_of` conflicts with 1207 (F3). Gettable flag and MCP arms unstated (F4, F5). |
| 1205 PharmacoDB | needs a fix, fixed and shipped | Acceptance 6 contradicted the measured sizes (F8). MCP arms unstated (F5). Both applied; shipped 2026-09-19. |
| 1206 DepMap | needs a fix, deferred | `data_as_of` format is defined three ways (F11). Acceptance 12 cannot hold as written (F12). MCP arms unstated (F5). Deferred 2026-09-19 over the DepMap terms contradiction; the findings are parked with the ticket. |
| 1207 assets and products | needs a fix | The `Public` comparison is wrong (F1). `data_as_of` conflicts with 1204 (F3). |
| 1208 dataset download | needs a fix | Manifest has no series-level date, so 1215 cannot work (F2). Lock PID order is racy (F13). Refresh conflict shape undefined (F14). |
| 1209 study import | needs a decision | The error-rendering change is global and belongs in its own ticket (F6). Array table reader unnamed (F17). |
| 1210 study compare | ready | No findings beyond its dependence on F6. |
| 1211 series section | needs a fix | Owns the fix for F1. Its payload has no `data_as_of` (F18). |
| 1212 study score | ready | Polish only (F5). |
| 1213 HPA cell lines | needs a fix, fixed and shipped | The fixture covered one of five Cellosaurus batches the spec needs (F16). Applied; shipped 2026-09-19. |
| 1214 ChEMBL section | ready, shipped | No findings. Shipped 2026-09-18. |
| 1215 dataset check | needs a fix | The `Public` comparison is wrong (F1). It never reads a header for the motivating case (F2). |
| 1216 dataset list | needs a fix | Shared lock on a file that hand-written fixtures do not have, and no wait bound (F15). |

Counts as reviewed on 2026-09-17: ready 4, needs a fix 10, needs a decision 1.

Disposition of the five cell line rows as of 2026-09-19: 1202, 1205, 1213 and 1214 took their fixes and shipped, and 1206 is deferred. The eleven dataset rows are unchanged and still describe the tickets as they stand.

## Blocking findings

### F1. `!Series_status` is never the bare word `Public`

Tickets: 1204, 1207, 1215, owned by 1211.

Problem. 1204 renders a warning when `status` "is not `Public`" (acceptance 8). 1207 acceptance 15 and the 1215 status table use the same rule, and 1215 reports `withdrawn` for it. 1211 returns the raw cell text and "normalizes no status word", and 1215 compares "as strings". Together these rules flag every public series.

Evidence. The measured header of GSE100446 reads `!Series_status "Public on Sep 11 2019"`. The submission and update lines read `"Jun 25 2017"` and `"Nov 04 2019"`. A byte comparison against `Public` fails on every public file.

Fix. 1211 defines one predicate beside the three fields: `status_is_public(raw)` is true when the trimmed raw value starts with `Public`. 1211 returns it as a fourth field, `is_public`, next to `status`. 1204, 1207, and 1215 test `is_public` and print the raw `status` verbatim. 1215 keeps HTTP 404 as its second `withdrawn` signal. 1211 acceptance 9 adds a fixture whose status is `"Public on Jan 01 2020"` and one whose status is another word, and pins the predicate on both.

### F2. `dataset check` never reads a header for a series with no matrix asset

Tickets: 1215, with a manifest change in 1208.

Problem. 1215 calls `read_header` "once per platform matrix URL the manifest's matrix assets name" and "builds no URL for an asset that is not a series matrix". The motivating case is an RNA-seq series such as GSE48843, whose matrix is empty and whose values live in an NCBI count file. A user downloads `ncbi:GSE48843_raw_counts_GRCh38.p13_NCBI.tsv.gz` and never the matrix. That manifest has no matrix asset, so `dataset check` sends no request, reports `unknown` on every row, and cannot report `withdrawn`. The proposal (section 2) says the command "re-reads the matrix header for each downloaded series".

Evidence. 1207 Current Facts: 654 of 774 matrix files have an empty table, 490 of them RNA-Seq. 1208 stores `upstream_last_updated` per asset and sets it to `null` for count and supplementary files. 1215 acceptance 4: a `null` asset "triggers no request for that asset". Nothing else in 1215 triggers one.

Fix. 1208 adds a series-level block to `manifest.json`: `series: {platforms: [...], upstream_last_updated, status, matrix_urls: [...]}`, filled from the 1207 scan that every download already runs. 1215 reads the header once per series from `matrix_urls`, whatever assets were downloaded. The `withdrawn` verdict is series-level and applies to every row. The date comparison uses the series-level value for every asset, since the series date is the only date GEO publishes for any of its files. Per-asset `unknown` then means only "the header could not be read". 1215 acceptance 4 changes to: a series whose only asset is an NCBI count file still gets one header read and a `current` or `changed_upstream` row. The proposal's line that `unknown` is the honest answer for count files should be retired in the same edit, because the series date does cover them.

### F3. `data_as_of` means three things across the GEO tickets

Tickets: 1204, 1207, 1208, 1209, with 1211 and 1215 affected.

Problem. 1204 defines `data_as_of` for "every dataset output" as the ESummary retrieval time and prints `last_updated` beside it as its own field. 1207 sets the `assets` and `products` payloads' `data_as_of` to `!Series_last_update_date` with `data_as_of_kind: "upstream_last_updated"`. 1208 and 1209 set it to `upstream_last_updated` when present and to `downloaded_at` otherwise, "with `data_as_of_kind` naming which", and never name the kind values. A single `get dataset GSE982 assets` payload therefore carries a top-level `data_as_of` that is a retrieval time and a nested one that is a 2019 header date, both under the same name. No ticket lists the `data_as_of_kind` enum. Across all fifteen tickets the values in use are `release`, `retrieved`, `upstream_last_updated`, and an unnamed one for `downloaded_at`.

Evidence. 1204 design "Dates on the card"; 1207 design "Dates and attribution"; 1208 design bullet "Every output carries `data_as_of`"; 1209 design "Output" bullet. Proposal section 7 defines `data_as_of` as the release name, the release date, or the retrieval time when the provider publishes neither, and section 1 keeps `last_updated` as a separate field.

Fix. Follow the proposal. 1204 owns the definition and states the closed enum for `data_as_of_kind`: `release`, `retrieved`. For GEO, `data_as_of` is always the retrieval time with kind `retrieved`, and `submitted`, `last_updated`, `status` travel as their own fields on every payload that read them. 1207 drops the `upstream_last_updated` kind and carries `last_updated` and `status` as named fields on the assets payload. 1208 and 1209 set `data_as_of` to `downloaded_at` and the import time respectively, both kind `retrieved`, and keep `upstream_last_updated` as the separate field they already store. The attribution lines then read "Retrieved <time>" everywhere. 1204 already prescribes that.

## Non-blocking findings that need a ticket edit

### F4. 1204 never flips the `dataset` entity to gettable

1203 registers `("dataset", true, false)` in `ENTITY_FLAGS` (`src/cli/list/catalog.rs:38`), and 1204 adds `get dataset` with sections but never says to change the flag to `true` or to add a section-name row in the catalog map at `src/cli/list/catalog.rs:78`. 1202 shows the full registration for `cell-line`; 1204 should say the same for `dataset`, and its acceptance should pin the catalog and the typed MCP `get` branch the way 1202 acceptance 12 does.

### F5. The MCP allow list is an exhaustive match, and most tickets do not say which arm they add

`is_allowed_mcp_command` (`src/mcp/shell.rs:472`) matches every `Commands` and subcommand variant by name and has no wildcard. Any new top-level command (`Dataset`, `CellLine`, `Depmap`) or new variant (`ArticleCommand::Datasets`, `GeneCommand::Dependency`, `GeneCommand::CellLines`, `DrugCommand::CellLines`, `StudyCommand::Import`, `StudyCommand::Score`) fails the build until it is classified. Tickets 1208, 1215, and 1216 state their rejections. 1209 and 1212 state theirs. 1204, 1205, 1206, and 1213 say the helpers "reach MCP through the raw tool" and stop there, and 1206 never says that `depmap sync` is rejected. Fix: each of 1204, 1205, 1206, and 1213 names the arm and the allow or reject verdict for every new variant, and 1206 rejects `depmap sync` with the local-write rule that `who-ivd sync` follows. The three dataset tickets each rewrite the same rejection message with their own wording; 1208 should own one `DATASET_LOCAL_MCP_REJECTION_MESSAGE` and 1215 and 1216 should extend it, not restate it. 1209 and 1212 should also say whether `GENERIC_MCP_REJECTION_MESSAGE` (`src/mcp/shell.rs:326`), which lists the allowed study commands by name, gains `study score --weights`.

### F6. The study error-rendering change in 1209 is global and should be its own ticket (needs a decision)

1209 design "Error rendering" changes how every `study` command failure prints, in Markdown and JSON, from `Source unavailable: cBioPortal DataHub is not available.` (`src/error.rs:482`, `:573`) to a message naming the file. That touches every existing study command, not the import. 1210 acceptance 8 depends on it, so 1210 cannot land before it. The string is not pinned in `spec/` or tests today, so the change is safe, but `spec/surface/cli-contract-ratchet.md` exists and the lead should confirm the ratchet accepts a changed error line. Recommendation: split it into a small ticket that 1209 and 1210 both depend on, so 1210 can start as soon as that ticket lands and does not wait for the whole import.

### F7. ESummary on `db=gds` does publish a date, and 1203 discards it

1204 states that ESummary "publishes no update date and no release name" and "no release name or release date for a series". The live GSE982 record carries `pdat: "2004/01/30"` (the date the series went public) and `ftplink: "ftp://ftp.ncbi.nlm.nih.gov/geo/series/GSEnnn/GSE982/"`. Neither is the update date, so the header read stays necessary, but `pdat` is free on the card. Fix: 1203's decoder keeps `pdat` as `published` on every row, 1204 prints it beside `submitted`, and 1204's claim narrows to "no update date". 1211 may take the matrix folder from `ftplink` rather than computing `series_prefix`, or keep the prefix rule and pin it against `ftplink` in a test. The same record has `relations: []`, so 1204's two-platform fixture with a SuperSeries relation must be a different series, not GSE982.

### F8. 1205 acceptance 6 contradicts the measured sizes

Acceptance 6 says a synthetic body over 32 MiB is "the doxorubicin and K-562 full-field case". The ticket's own facts say K-562 is 19.4 MB and doxorubicin 18.6 MB, both under the 32 MiB cap the ticket sets. Those two are the reason the cap is 32 MiB, and both must pass. Fix: acceptance 6 keeps the over-cap synthetic body as a hypothetical and adds a 19.4 MB synthetic full body that parses. The 32 MiB decision line should say that no measured input exceeds it.

### F9. 1202 opts out of `section_outcomes`, and its dependents opt back in

1202 says the requested sections "widen the `fields` list of the one record request, so no section makes an extra call and the entity needs no `section_outcomes` registry row". 1205 adds `drug_response` "with a matching outcome key", 1214 adds `chembl` "with an outcome key", and 1206 adds `depmap` with an `unavailable` outcome. The pathway entity is the model 1202 names, and it carries `section_outcomes` (`src/entities/pathway.rs:18`), and `spec/entity/section-outcomes.md` ties `_meta.section_sources` to it. Fix: 1202 adds the `section_outcomes` field and the `_meta.section_sources` wiring with `variants` and `xrefs` as `data` or `empty`, so 1205, 1206, and 1214 only add keys.

### F10. 1202's hyphen rule needs a fixture it does not list

The design runs two searches when the query contains a hyphen. `MOLM-13` therefore sends `idsy:"MOLM-13"` and `idsy:"MOLM13"`, and acceptance 2 requires the `MOLM-13` search to succeed. The fixture list has `search_idsy_molm13` only, and the fixture server answers an empty result only for unknown `dr:` queries. Fix: record `search_idsy_molm_13_20260917.json`, or have the fixture server answer an empty window for any unlisted `idsy:` query and say so.

### F11. 1206 defines the `data_as_of` string three ways

The design says `data_as_of` is `<release title> (<published_date>)`, gives `24Q4 (2024-12-10)` as the example, and writes the attribution line as `DepMap 24Q4 (2024-12-10)`. The release title on Figshare is `DepMap 24Q4 Public`, so the literal rule yields `DepMap 24Q4 Public (2024-12-10)`. Fix: name the captured `\d{2}Q[1-4]` group from the release regex as `release_tag`, define `data_as_of` as `<release_tag> (<published_date>)`, and use the same tag in the attribution line and in the no-screen messages.

### F12. 1206 acceptance 12 cannot hold as written

It says that with no install "the sections and the helpers report unavailable with the exact message and make no request". `get gene KMT2A dependency` resolves the symbol through the existing gene lookup first. That lookup is a network request. Fix: "make no Figshare or DepMap request", and say whether the symbol resolves before or after the install check.

### F13. 1208's lock takes the PID after the race

The design says "the holder writes its process ID into the lock file before it takes the lock". Two writers racing both write, and the loser may overwrite the winner's PID, so a blocked third writer names the wrong process. Fix: take the lock first, then truncate and write the PID; a waiter that times out reads the file after the wait. Acceptance 16 then holds.

### F14. 1208's refresh conflict has no manifest shape

"It keeps the old file as `<name>.<old sha prefix>`, records both, and reports the conflict", while the manifest "gains or replaces one entry" per asset. Nothing says where the second record lives. 1216 lists assets from the manifest and has no state for a superseded file. Fix: the entry gains `previous: [{path, bytes, sha256, downloaded_at}]`, 1216 prints those rows with state `superseded` and counts their bytes in the total, and 1209 records which SHA-256 it imported so `dataset list` can show which version a study used.

### F15. 1216's shared lock needs a file that its fixtures do not have

1216 reads each manifest under `manifest.lock` in shared mode and "writes nothing, not even the lock file's PID line", and acceptance 5 requires the tree unchanged after the run. Its fixtures are hand-written manifests. A missing lock file cannot be locked without creating it. Fix: state that a series folder with no `manifest.lock` is read without a lock, and put a `manifest.lock` in the fixtures that test locking. Also bound the wait the way 1208 does (30 seconds, then a message naming the holder), because acceptance 10 says only "wait".

### F16. 1213's fixture covers one of the five Cellosaurus batches the spec needs

The join runs one Cellosaurus search per batch of 20, so the 91-line leukemia group needs five requests. The fixture list records one batch. The spec block requires `total == 91` and the MOLM-13 accession, and acceptance 3 requires OCI-AML-3 too. Fix: record all five batches, or say that the fixture server answers an empty window for any unlisted `id:` query and that an empty window is a `null` accession, not the "Cellosaurus failure" note of acceptance 4.

### F17. 1209 names no reader for the array value table

1211's `read_header` stops at `!series_matrix_table_begin`. 1209 reads the table "past the header" for arrays and says nothing about how. The table has quoted GSM headers, blank cells (15 of 120 files with values), values at or below zero, and up to 5,456,443 rows. Fix: name a streaming table reader in `geo_matrix.rs` that reuses the header parser, drops the quotes, treats a blank cell as missing, and pins one fixture with a blank cell and one with a negative value.

### F18. 1211's `series` payload carries no `data_as_of`

1204 says every dataset output carries `data_as_of` and the attribution line. 1211's JSON shape is `series: {platforms: [...]}` with no such field and its acceptance never mentions one. Fix: 1211 says the section rides on the card's top-level `data_as_of` and adds that to acceptance 10.

## Polish

### F19. 1203's JSON example omits two fields its design adds

The decoder produces `supplementary_types` and `geo2r`, and 1204 renders both. The JSON example in 1203 does not show them. Add them so the fixture test pins them.

### F20. 1208 lists four asset ID kinds; 1207 defines six

1208 Current Facts names `matrix:`, `annot:`, `ncbi:`, and `suppl:`. 1207 also defines `gene_info:<organism>` and `gene_history`. Those two are not series files. 1208's layout puts every asset under `<root>/geo/<GSE>/`, so a user importing three RNA-seq series stores three 162 MB copies of `gene_history.gz`. Fix: 1208 stores `gene_info:` and `gene_history` once under `<root>/ncbi-gene/` with their own manifest, and 1209 resolves `--annotation` there.

### F21. 1208 re-runs the whole 1207 scan before every download

`download` "resolves the asset by re-running the 1207 scan", which reads every platform header, the counts page, the `suppl/` listing, and one HEAD per platform. Acceptance 2 and 5 say `--dry-run` and a repeat download make "no request for the asset URL". That is true. A reader may still take it as no request at all. State the scan requests, and let `path` and a repeat download of a manifest-listed asset skip the scan.

### F22. 1207 calls a product `usable_values`

The `status` value `usable_values` reads as a fitness claim, and the same paragraph says "BioMCP makes no claim that any file suits an analysis". Rename it `values_present`; `meaning_unknown` can stay.

### F23. 1207 and 1211 say 588 series; the findings say 585

Experiment 203 counts 774 files over 585 series. 1207 and 1211 write 588. 1203 writes 585. Use one number.

### F24. 1206 hard-codes the model count

The index description says "one block of 1,178 values per gene". The count belongs to the 24Q4 file. Say "one block of `model_count` values", with the count from `gene_effect_axes.json`.

### F25. 1206 was asked for an ADR and gives a Decisions bullet

Survey finding D14 asks that the 21-month-old release limit be recorded in an ADR. The ticket records it under Decisions. Either is durable; say which one the repo wants, since `sdlc/records/` and this note are the only records today.

## Contradictions with the measured findings

Only F1, F7, and F8 contradict a measurement. Every other measured consequence in experiments 203 and 204 is absorbed: the 4 MiB line cap and line-feed splitting (1211), `has_values` with any first column name (1207), the gzip content check (1207, 1208), `gene_info` mapping and the retired and replaced split (1209), max-mean over kept samples with byte-order ties (1209), the three unit fields (1207, 1209, 1212), the raw-counts warning and `--allow-missing` reporting (1212), the Mann-Whitney method statement and R pin (1210, 1212), distinct GSM tallies (1204, 1211), the KG1 and NB4 ranking facts (1202), the `dr` cost (1202), the PharmacoDB size guard and name fallback (1205), the RRID list join and no-screen message (1206), and the PRISM follow-up retirement (1206).

## Code citations

I checked 154 `file:line` citations across the fifteen tickets against `src/`, `docs/`, `spec/`, `scripts/`, `architecture/`, and `Cargo.toml` at `45d74711`. Every one lands on the named symbol or within five lines of it. The widest drifts are `src/sources/figshare.rs:89` (struct at `:90`), `src/error.rs:481` and `:572` (match arms at `:482` and `:573`), `src/cli/article/session.rs:286` (`create_temp_path` at `:285`), and `mkdocs.yml:43` (the `Sources:` block starts at `:41`). The `mann_whitney_u_test` citation that experiment 203 flagged is already `:1919` in 1210 and 1212. No citation is wrong.

## Scope and boundaries

Nothing crosses the line. 1209's max-mean probe rule is a documented mechanical choice recorded in `meta_study.txt`, with the rule name as a flag. 1206 sorts by gene effect and cuts at N. Its own text allows that as "sort order". 1204's docs guidance that sample maps usually come from `treatment`, `agent`, and titles is advice on where to look. It labels nothing. The one naming slip is F22.

## Buildability

Questions an engineer would have to ask, beyond the findings above:

- 1204: does `get dataset X series assets` in one invocation fill the card fields from the `series` reader or the `assets` scan, and do they share one header read? (Both tickets say the scan is shared; neither names the owner.)
- 1208: what does `--dry-run` print for a resumable `.partial` file, and does `--no-resume` need `--refresh`?
- 1209: which SHA-256 goes into `import.json` when the manifest entry has a `previous` list (F14)?
- 1215 and 1216: what is `role` for a `previous` file row, and does `--limit` count skipped series with unparsable manifests?
- 1213: is the HPA `search` parameter an Ensembl ID or a symbol in the recorded fixture? The design says Ensembl ID; the observed call used `FLT3`.

## Sequencing

The `deps` chains are right: 1203 → 1204 → 1211 → 1207 → 1208 → 1209 → 1210 → 1212, with 1215 on [1208, 1211] and 1216 on [1208, 1209]; and 1202 → {1205, 1206, 1213, 1214}. Two adjustments:

- Split the study error rendering out of 1209 (F6) so 1210 can start after that small ticket instead of after the whole import.
- 1208 carries six features from the proposal (data root, layout, download, disk floor, lock, resume, refresh conflict, manifest, path, MCP) with 19 acceptance items. Resume (proposal section 10) is independent new code with its own HTTP semantics and no other ticket needs it. Move it to a follow-up that depends on 1208. The lock stays, because 1209 appends to the manifest.

1216 depends on 1209 only for the imported-studies line and spec block 13. It can be built after 1208 with an empty `imports` list and re-verified after 1209.

## Disposition, 2026-09-17

Ian approved every finding. All 25 are applied to the tickets, together with both splits.

- Resume moved out of 1208 into ticket 1217, which depends on 1208.
- The study error rendering moved out of 1209 into ticket 1218. Ticket 1209 now depends on [1208, 1218] and ticket 1210 on [1218] alone, so the grouped compare no longer waits for the import.
- F2 retires one line of the proposal. `sdlc/planning/2026-09-17-dataset-storage-and-freshness.md` section 2 and decision 4 are amended in the same commit: the check is per series, and `unknown` now means only that the header could not be read.
- F25 is recorded as a Decisions bullet in 1206, not an ADR. The repo writes an ADR when a decision reverses a recorded product boundary.
- One fact the review did not have: the shared `SourceUnavailable` renderer also serves DDInter and MyDisease.info, whose exact strings are pinned at `spec/entity/section-outcomes.md:175`, `:177`, and `spec/entity/diagnostic.md:116`. Ticket 1218 therefore adds an optional `detail` field instead of changing the shared sentence.
