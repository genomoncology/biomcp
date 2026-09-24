# BioMCP ideas from KIDS26, scored

Written 2026-09-16 from the St. Jude KIDS26 BioHackathon and moved here from Ian's notes on 2026-09-24. Tickets 1202 to 1212 carry the result. This file keeps the rubric, the scores, and Ian's rulings behind them.

Nine candidate ideas, one rubric, one plan. Sources read: `repos/biomcp/architecture/ux/cli-reference.md`, `architecture/technical/source-integration.md`, `docs/reference/source-licensing.md`, `src/sources/ncbi_efetch.rs`, `src/mcp/catalog.rs`, `testdata/sources/`, and the BioMCP ideal state. Nothing in the repo mentions GEO, Cellosaurus, LINCS, DepMap, PRISM, GDSC, or PharmacoDB today.

## What the repo says the contract is

- The proposed dividing line "lookup and describe versus analysis" is not what the ideal state says. BioMCP already runs cohort, survival, comparison, and co-occurrence over cBioPortal bundles installed on disk (`biomcp study query`, `top-mutated`). Descriptive computation over published data is in scope.
- The lines the ideal state does draw: no interpretation (never scores a match or classifies), no curation (adds no knowledge of its own), no mirroring of upstream datasets, no account required for any command, and every printed identifier types back in as an input.
- The usable rule for this list: BioMCP may look up, describe, and count things that already carry a public identifier. It may not assign a label or a score that a paper would report as a result.
- Adding a source is one module in `src/sources/`, one entity folder under `src/entities/` and `src/cli/`, one row in `source-licensing.md` and `sources.json`, and JSON fixtures in `testdata/sources/<source>/`. MCP exposes generic `search` and `get` tools, so a new entity adds a type, not a tool.
- The NCBI client only wraps `efetch` for PubMed and ClinVar. A generic `esearch`, `esummary`, `elink` helper does not exist and is a prerequisite for any GEO work.

## Rubric

Each criterion scores 0 to 5. Weights sum to 100. Higher is always better, so cost and creep are scored as "how cheap" and "how contained".

| Criterion | Scale (0 to 5) | Weight | Why this weight |
| --- | --- | --- | --- |
| Demand beyond this project | 0 = only Team 19 asks; 5 = a common question any biomedical agent gets | 15 | BioMCP serves every agent session, not one hackathon |
| Entity validity | 0 = no identifier; 5 = stable public accession that pivots to existing entities | 12 | The three-verb grammar needs a thing with an id |
| Contract fit | 0 = interpretation or curation; 5 = lookup and describe with no judgment | 15 | The ideal state's non-goals are hard rules |
| Source terms and stability | 0 = research-only or unclear; 5 = open license, no account, funded institution | 12 | BioMCP promises no account and no paid service, and its docs publish per-source terms |
| Access shape | 0 = large heterogeneous bulk; 5 = small JSON over a public API | 8 | Determines whether a request is on demand or an install |
| CLI and MCP clarity | 0 = no clean command; 5 = fits `search`, `get`, or a pivot helper without a new verb | 8 | A feature that needs a new verb costs the whole grammar |
| Build and maintenance cost | 0 = a pipeline; 5 = one source module and one entity | 10 | Ian runs this channel by hand |
| Testability | 0 = needs live data; 5 = a few recorded JSON fixtures | 5 | Gates run offline against `testdata/` |
| Cross-entity leverage | 0 = a dead end; 5 = pivots to or from three or more existing entities | 10 | Pivoting without rebuilding a query is the product |
| Scope containment | 0 = pulls the next three ideas in; 5 = stops cleanly | 5 | Creep is the main way a lookup tool becomes a pipeline |

## Scores

| # | Idea | Demand | Entity | Contract | Terms | Access | CLI | Cost | Test | Leverage | Contain | Score |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 3 | Cell line entity (Cellosaurus) | 5 | 5 | 5 | 5 | 5 | 5 | 4 | 5 | 4 | 4 | 95 |
| 1 | GEO dataset search | 5 | 5 | 5 | 5 | 5 | 5 | 4 | 5 | 4 | 3 | 94 |
| 2 | GEO dataset details and article link | 4 | 5 | 4 | 5 | 3 | 5 | 3 | 5 | 5 | 2 | 84 |
| 6 | PharmacoDB / GDSC | 3 | 3 | 4 | 2 | 4 | 4 | 3 | 4 | 5 | 3 | 69 |
| 5 | DepMap / PRISM | 4 | 3 | 3 | 3 | 2 | 3 | 2 | 3 | 5 | 2 | 62 |
| 4 | LINCS / CLUE | 4 | 3 | 2 | 1 | 2 | 3 | 2 | 3 | 4 | 1 | 52 |
| 7 | Expression table parsing, GPL mapping | 3 | 2 | 1 | 5 | 1 | 2 | 1 | 2 | 2 | 0 | 42 |
| 8 | Treatment versus control labeling | 3 | 1 | 0 | 5 | 3 | 1 | 1 | 2 | 1 | 0 | 36 |
| 9 | LSC6/LSC17 scoring and drug ranking | 1 | 1 | 0 | 4 | 3 | 1 | 2 | 3 | 1 | 1 | 31 |

Justifications:

- **3, cell line.** CVCL accessions are stable, Cellosaurus is CC BY 4.0 with a keyless REST API (api.cellosaurus.org), and one record carries disease, species, and cross-references to DepMap, cBioPortal CCLE, ChEMBL, and GDSC. It is the key every bigger-picture source joins on. Tier 1 in the licensing registry.
- **1, GEO dataset search.** GSE accessions are stable. `esearch` and `esummary` with `db=gds` return title, organism, type, platforms, sample count, and PubMed ids in one JSON call at 3 requests per second without a key, 10 with. Loses one point on containment because a dataset search invites a download command.
- **2, GEO details.** `esummary` already returns sample accessions and titles, so a first version needs no file read. Characteristics need the matrix header, which is gzip on FTP and must be streamed and cut at the table marker. The elink from PubMed to GDS is cheap and fits the existing `article` entity as a pivot. Contained poorly because the header sits next to the expression table.
- **6, PharmacoDB and GDSC.** PharmacoDB has a keyless REST API keyed on drug, cell line, and dataset, which is a lookup shape. Its data license could not be found (code is GPL-3.0). GDSC's own terms allow internal research only and exclude commercial services, and its legal page returned 410. Terms are the blocker, not the shape.
- **5, DepMap and PRISM.** Bulk download only, quarterly releases, no API. DepMap terms say CC BY 4.0 and also "not intended for clinical or commercial uses" on the same page. Fits the `study` install pattern, which the ideal state already tolerates, but "mirrors no upstream dataset" is a stated tenet and Ian owns that ruling.
- **4, LINCS and CLUE.** Terms are free for academic and non-profit users only, the API needs a personal key tied to a registered user, keys and data may not be redistributed, and commercial use requires contacting CLUE. That conflicts with "no command requires an account" and with GenomOncology as the maintainer. Connectivity queries are also analysis. Lookup of "which signatures exist for drug X" would be tier 3 at best.
- **7, expression parsing.** No identifier to look up. Most candidate RNA-seq series carry no table in the matrix file, so the parser would need one adapter per lab format. R's GEOquery already does the array case. Cost and containment fail even though the terms are fine.
- **8, treatment versus control labeling.** Assigning a label to free text is curation, which the non-goals forbid. It is also the team's day 3 research. A future descriptive helper could print the raw characteristics lines per sample and stop there, which is idea 2.
- **9, LSC scoring.** Scoring is interpretation, the research question is Nobel's, and demand outside AML is near zero.

## Grouping

| Good fits | Bigger picture | Does not fit |
| --- | --- | --- |
| 3 Cell line entity | 6 PharmacoDB / GDSC | 7 Expression parsing |
| 1 GEO dataset search | 5 DepMap / PRISM | 8 Treatment versus control labeling |
| 2 GEO dataset details and article link | 4 LINCS / CLUE | 9 LSC scoring and ranking |

The three-by-three grouping stands. The evidence changes the order inside two groups: cell line moves ahead of GEO search because it is the cheapest, the best licensed, and the join key for the whole bigger-picture group. LINCS drops to the bottom of bigger picture on terms and may never enter.

## Where this disagrees with the original recommendations

- **The dividing line.** "Lookup and describe versus analysis" is not BioMCP's rule. The `study` entity already computes over local bundles. The rule that holds is no interpretation, no curation, no account. Ideas 7 to 9 still fail, but 7 fails on cost and heterogeneity, 8 on curation, and 9 on interpretation. Saying so keeps future descriptive features (counts, sample tables) inside the fence.
- **Idea 2 is not "kilobytes".** The matrix file is gzip on FTP. Reading the header means streaming and cutting at `!series_matrix_table_begin`, with a byte cap and a fixture from a multi-platform series. It is a real step with its own ticket, and the `esummary` sample list should ship first without it.
- **Idea 1 needs a prerequisite.** A generic E-utilities helper (`esearch`, `esummary`, `elink` with any `db`) does not exist. That refactor is the first GEO ticket and also pays off for PubMed.
- **LINCS is a terms problem before it is a product problem.** The original list treats it as a data source to weigh. Its terms clash with two tenets. It belongs with Ian's rulings, not the backlog.
- **The `study` name is taken.** `study` means a cBioPortal DataHub bundle with local install semantics. A GEO series should not share that entity. Proposed name: `dataset`.

## Command shapes

Checked against the `search <entity>`, `get <entity> <id> [section]`, and `<entity> <helper> <id>` grammar in `cli-reference.md`. Entity names use underscores today (`adverse_event`).

```
biomcp search cell_line MOLM13
biomcp get cell_line CVCL_2119
biomcp get cell_line CVCL_2119 xrefs
biomcp search dataset --disease "acute myeloid leukemia" --organism human --keyword DMSO
biomcp get dataset GSE982
biomcp get dataset GSE982 samples
biomcp article datasets 12345678
biomcp cell_line datasets CVCL_2119
```

The last two are pivots and follow the `variant trials` pattern. MCP needs no new tool: `search` and `get` take a new entity type.

## Plan

**During the hackathon, build nothing in biomcp.** Ian mentors. The team writes its own GEO discovery in R because Nobel wants the team to learn it. Ian's only BioMCP action is to save evidence: which E-utilities queries the team ran, which fields they needed, which spellings of cell lines appeared, and one raw `esummary` JSON and one matrix header as future fixtures. That evidence lands in the team repo or in Ian's hackathon notes, not in biomcp.

**Step 1: cell_line entity from Cellosaurus.** Gate: one ticket in `sdlc/tickets/`, a licensing row (tier 1, CC BY 4.0, attribution in output), three recorded fixtures (MOLM13, MV4-11, one ambiguous name). No demand gate: cell lines already appear in cBioPortal CCLE studies, GDSC, and ChEMBL records that BioMCP serves, so the pivot exists today.

**Step 2: generic E-utilities helper, then dataset search.** Gate: the helper refactor lands first with the existing PubMed and ClinVar callers moved onto it. Then `search dataset` and `get dataset` from `esummary`, plus the `article datasets` pivot from `elink`. Licensing row for NCBI (tier 1, no restrictions from NCBI, submitter rights noted). Demand gate: the hackathon evidence plus one question from outside Team 19, from an agent evaluation set or a user.

**Step 3: dataset samples from the matrix header.** Gate: step 2 shipped and used, a byte cap on the streamed read, a fixture from a multi-platform series (one file per platform), and the output printing raw characteristics lines with no labeling. This is where the fence sits. Nothing past the table marker is ever read.

**Step 4: bigger picture, only after step 1.** Order: PharmacoDB first because it is an API. Gate: its data license confirmed in writing on its site or from its maintainers; Ian decides whether to ask. DepMap second, modeled on `study download`. Gate: Ian's ruling on the mirroring tenet and on the commercial-use wording. LINCS last. Gate: Ian's ruling on a keyed, non-redistributable, research-only source. If that ruling is no, the idea is closed.

**Stays in the team repo:** every download of a full matrix file, supplementary count parsing, probe-to-gene mapping, treatment and control labeling, LSC6 and LSC17 scoring, and drug ranking. The medallion layout on Ian's fork is the right home for all of it.

**What Ian should not do during the event:** build any of the steps above, demo BioMCP to Team 19 as a replacement for their discovery step, or commit hackathon-derived fixtures into biomcp before the event ends.

## Decisions

Made by default in this note and open to Ian's overturn:

- The rubric weights and the resulting order (cell line first, then GEO search, then GEO details).
- The entity name `dataset` for GEO series instead of extending `study`.
- The demand gate on step 2 and the byte cap and no-labeling rule on step 3.
- PharmacoDB ahead of DepMap in the bigger-picture order.

Ian's to make, because they are outward-facing or contradict a tenet:

- Whether BioMCP ever takes a keyed, research-only source (LINCS). This touches the no-account promise and GenomOncology's commercial position.
- Whether a DepMap install is allowed under "mirrors no upstream dataset", given that `study download` already does this for cBioPortal.
- Whether to contact PharmacoDB about its data license.
- Whether BioMCP appears in the Team 19 demo at all.

## Ian's rulings, 2026-09-16

- Licenses do not block this work. BioMCP is open-source and non-commercial. PharmacoDB publishes its code under GPL-3.0 and its API and data under CC BY-NC 4.0.
- LINCS is skipped because it needs a personal key.
- Nothing from this work goes into the Team 19 repo or demo.
- Ideas 1 to 5 become BioMCP tickets 1202 to 1206.
- DepMap installs from its Figshare mirror. The portal download API returns a bot check to scripts. The 24Q4 release has 73 files totaling 30.83 GB. The default install needs `Model.csv` (0.6 MB, 2,105 models, with Cellosaurus RRIDs) and `CRISPRGeneEffect.csv` (428.7 MB, 1,178 models by 17,916 genes, 38 seconds to download, 206 MB compressed).
