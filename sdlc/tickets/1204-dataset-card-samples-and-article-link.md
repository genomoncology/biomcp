---
flow: build
priority: 3
deps: [1203]
---

# 1204: Dataset card, samples, and article link

## Goal

An agent opens one GEO series, lists its samples, and finds the GEO series linked to a PubMed article. Nothing in this ticket reads a file.

```
biomcp get dataset GSE982
biomcp get dataset geo:GSE982 publications links
biomcp dataset samples GSE982 --limit 25
biomcp article datasets 12345678
```

The motivating consumer is a hackathon team screening public GEO studies of drug-treated AML cell lines. The team needed the sample list before it downloaded any expression data.

## Current Facts

- Ticket 1203 adds the `dataset` entity and `search dataset`. This ticket adds `get dataset <GSE>`. The entity name `dataset` avoids `study`, which already names the cBioPortal DataHub command (`src/cli/commands.rs:78`).
- Ticket 1203 registers `("dataset", true, false)` in `ENTITY_FLAGS` (`src/cli/list/catalog.rs:38`), searchable and not gettable. This ticket flips the third flag to `true` and adds a `"dataset" => DATASET_SECTION_NAMES` row to the section-name map (`src/cli/list/catalog.rs:78`). The typed MCP `get` tool derives its entity list and section enum from that catalog (`typed_get_capabilities`, `src/mcp/shell/typed_get.rs:12`), so the new branch appears with no tool change. Ticket 1202 does the same registration for `cell-line`.
- `is_allowed_mcp_command` (`src/mcp/shell.rs:472`) matches every `Commands` and subcommand variant by name and has no wildcard arm. A new top-level command or a new subcommand variant fails the build until it is classified there.
- No code calls `elink` today (`grep -rn elink src` finds nothing). The PubMed client wraps only `esearch` and `esummary` for PubMed (`src/sources/pubmed.rs:338`, `src/sources/pubmed.rs:410`) against `BIOMCP_PUBMED_BASE` (`src/sources/pubmed.rs:12-14`). The generic E-utilities helper from 1203 gains `elink` here.
- Entities parse named sections with a local `parse_sections` (`src/entities/pathway.rs:121`). Article helpers live in `ArticleCommand` (`src/cli/article/mod.rs:308`). `article entities <pmid>` (`:328`) is the model for a PMID-keyed pivot with `--limit`.
- `all` is the default surface, not every opt-in section (`architecture/ux/cli-reference.md:97-104`).
- The source registry already has an `ncbi-e-utilities` row (`docs/reference/sources.json:979`). 1203 adds the GEO source page.

GEO facts the design relies on, to confirm against the recorded fixtures:

- `esummary` with `db=gds` returns, per series, `accession`, `entrytype`, `gpl`, `n_samples`, `bioproject`, `relations`, and a `samples` array of `{accession, title}`.
- Experiment 203 found 5,912 sample IDs in more than one of 585 series (774 files). A SuperSeries repeats the samples of its SubSeries. GSE982 is a SubSeries of GSE995.
- In the same survey, sample characteristics and sample titles were the first field that split samples by a treatment word in 569 of 623 files. The characteristics keys that split samples most often were `treatment` (336 series) and `agent` (41). The treatment protocol mentioned DMSO in 328 series and was usually the same for every sample.
- `elink` with `dbfrom=pubmed&db=gds` returns GDS UIDs of mixed record types (series, platforms, samples, curated DataSets).
- ESummary on `db=gds` publishes no update date and no release name. It does publish `pdat`, the date the series went public: the live GSE982 record read `pdat: "2004/01/30"` and `ftplink: "ftp://ftp.ncbi.nlm.nih.gov/geo/series/GSEnnn/GSE982/"` on 2026-09-17. Ticket 1203 keeps `pdat` as `published` on every row. The provider's own update date and status live in the series matrix header as `!Series_submission_date`, `!Series_last_update_date`, and `!Series_status`. Ticket 1211 returns them as `submitted`, `last_updated`, `status`, and the predicate `is_public`. The GEO download service sends no `ETag` and no `Last-Modified`, so the header is the only update signal GEO offers.
- `!Series_status` is a sentence, not a word. The measured GSE100446 header reads `!Series_status "Public on Sep 11 2019"`, so a comparison against `Public` flags every public series. Ticket 1211 owns the `is_public` predicate, which is true when the trimmed raw value starts with `Public`.
- The live GSE982 record carries `relations: []`, so it cannot serve as the two-platform SuperSeries fixture.

## Design

### `get dataset <id>` card

`get dataset <id>` accepts `geo:GSE…` or bare `GSE…` and renders the 1203 row for that series from one `esummary` call: ID, title, organisms, series types, platforms, sample count, PMIDs, supplementary types, `geo2r`, `published`, and summary. Sections `publications` (the PMIDs with `get article` next commands) and `links` (the GEO page, BioProject, and SuperSeries/SubSeries relations) read the same response. The card names the SuperSeries or SubSeries relation when one exists.

`all` means the sections that make no file request: the card, `publications`, and `links`. It never includes `series` (ticket 1211) or `assets` and `products` (ticket 1207).

### Registration

- `ENTITY_FLAGS` (`src/cli/list/catalog.rs:38`) changes to `("dataset", true, true)`.
- The section-name map (`src/cli/list/catalog.rs:78`) gains `"dataset" => DATASET_SECTION_NAMES`, holding `publications`, `links`, and `all`. Tickets 1211 and 1207 add `series`, `assets`, and `products` to that list.
- Typed MCP `get` picks the entity and its sections up from the catalog, so its schema gains a `dataset` branch with no tool change. Typed MCP `search` stays at eight entities, and `search dataset` reaches MCP only through the raw tool.

### MCP arms

`is_allowed_mcp_command` (`src/mcp/shell.rs:472`) is an exhaustive match, so every new variant is classified here or the build fails.

- `Commands::Dataset` is new. Its `DatasetCommand::Samples` arm is allowed: it makes one bounded ESummary request and reveals no local path. Every other `DatasetCommand` arm is added by a later ticket and rejected there.
- `ArticleCommand::Datasets` is allowed, like the other article helper families.
- Nothing in this ticket reads or writes local state, so `GENERIC_MCP_REJECTION_MESSAGE` (`src/mcp/shell.rs:326`) is unchanged.

### Dates on the card

`data_as_of` means one thing across every BioMCP output: when BioMCP fetched the data, or the provider's own release when it publishes one. It is never the provider's record-update date. `data_as_of_kind` is a closed enum with two values:

| Kind | Meaning |
| --- | --- |
| `release` | The provider publishes a release name or a release date, and `data_as_of` holds it. |
| `retrieved` | The provider publishes neither, and `data_as_of` holds the time BioMCP fetched the data. |

For NCBI GEO the kind is always `retrieved`, because GEO publishes no release name and no release date for a series. Every dataset output carries `data_as_of` and `data_as_of_kind` at the top level of the payload, including `dataset samples` and `article datasets`, and every dataset ticket that follows uses the same two values. GEO's own dates never fill `data_as_of`. They travel as their own named fields on any payload that read them: `published` from ESummary `pdat`, and `submitted`, `last_updated`, `status`, and `is_public` from the matrix header.

The card therefore carries these fields beyond the 1203 row: `published`, `submitted`, `last_updated`, `status`, `is_public`, `data_as_of`, and `data_as_of_kind`.

- `published` comes from the ESummary `pdat` member and prints beside `submitted`. It is the date the series went public and is not an update date.
- `submitted`, `last_updated`, `status`, and `is_public` come from the series matrix header through ticket 1211's `read_header`. The card alone reads no file, so a plain card renders them as `unknown` and prints one note: `Provider dates come from the series matrix header. Run biomcp get dataset <id> series.` When the invocation also asks for `series`, or for `assets` (ticket 1207), the scan already read the header and the card prints the values as published.
- An `is_public` of `false` renders a warning line above the card body, naming the raw status verbatim, for example `Warning: GEO status is "Withdrawn".` The line prints in Markdown and the raw value sits in JSON as `status`. BioMCP normalizes no status word, compares no status to a word, and applies no threshold. An `is_public` of `null`, which is a card with no header read, renders no warning.
- Markdown prints the attribution line `NCBI GEO records are public; submitters keep rights to their data. Retrieved <data_as_of>.`

### `dataset samples <id>` helper

`biomcp dataset samples <id> --limit 25 --offset 0` renders the `samples` array from the same `esummary` call. It is a helper, like `article entities`, because series can have hundreds of samples and need paging. `--limit` accepts 1 to 500. It makes no extra request. Output is a table of sample accession and title in provider order, plus `n_samples`. JSON adds `samples: [{accession, title}]`. Sample accessions print as bare `GSM` strings.

### `article datasets <pmid>` pivot

`ArticleCommand::Datasets { pmid, limit }` with `--limit` 1 to 50, default 10.

- The command calls `elink` with `dbfrom=pubmed&db=gds&id=<pmid>`, then `esummary` with `db=gds` on the linked UIDs, then keeps rows whose `entrytype` is `GSE`. It renders the same compact row that `search dataset` renders.
- No links gives an empty result with the note `No GEO series linked to PMID <pmid>.` and exit 0.
- Each row carries the next command `biomcp get dataset geo:<GSE>`. The empty result suggests `biomcp search dataset -k <pmid>`.
- Help text: `Find GEO series linked to one PubMed article`, with two examples and `See also: biomcp list article`.

### Docs

- Add the card, `publications`, `links`, the `dataset samples` helper, and the `article datasets` row to the GEO source page.
- State the `data_as_of` definition and the two-value `data_as_of_kind` enum once, on the GEO source page and in `docs/reference/sources.json` prose, and link both from `docs/user-guide/cli-reference.md`. Say that `data_as_of` is when BioMCP fetched the data, that GEO's kind is always `retrieved`, and that GEO's own dates are the separate `published`, `submitted`, and `last_updated` fields.
- Add the sections, the helper, and the pivot to `biomcp list dataset` and `biomcp list article`.
- State that a SuperSeries repeats its SubSeries samples. Any tally across series counts distinct GSM IDs.
- State that sample maps for ticket 1209 usually come from characteristics keys such as `treatment` and `agent` and from sample titles. The treatment protocol rarely tells samples apart.

## Acceptance

Fixtures under `testdata/sources/geo/`, small and recorded: `esummary` JSON for one single-platform series with three samples (GSE982, whose `relations` is empty and whose `pdat` is `2004/01/30`) and one two-platform series with a SuperSeries or SubSeries relation, which is a different accession because GSE982 has none, and `elink` JSON for one PMID linking to a series, a platform, and a sample, plus one PMID with no links.

1. `get dataset GSE982` and `get dataset geo:GSE982` render the same card.
2. `publications` and `links` render from the fixture with no extra request. The two-platform fixture shows its SuperSeries or SubSeries relation on the card and in `links`. The GSE982 fixture shows no relation.
3. `dataset samples` renders samples in provider order and pages with `--offset`.
4. `article datasets` keeps only `GSE` rows from the mixed `elink` fixture and returns the empty note for the unlinked PMID.
5. `get dataset <GSE>` and `get dataset <GSE> all` send only the `esummary` request.
6. Every dataset output carries `data_as_of` and `data_as_of_kind: "retrieved"` in JSON and the `Retrieved <data_as_of>` attribution line in Markdown, including `dataset samples` and `article datasets`. The time comes from an injected clock so the test pins an exact string. No dataset payload sets `data_as_of` from a GEO date.
7. A plain card renders `submitted`, `last_updated`, `status`, and `is_public` as `unknown` with the note naming `get dataset <id> series`, and makes no matrix request. The card renders `published` as `2004-01-30` from the fixture `pdat`.
8. A fixture whose `is_public` is `false` renders the warning line with the raw status verbatim, in Markdown and in JSON, and a fixture whose status is `Public on Sep 11 2019` renders no warning. The renderer tests `is_public` and never compares `status` to a word. Ticket 1211 supplies both values; until it lands the test drives the renderer directly.
9. The catalog lists `dataset` as searchable and gettable with sections `publications`, `links`, and `all`. The typed MCP `get` schema gains a `dataset` branch. Typed MCP `search` still rejects `dataset`.
10. The MCP shell allows `dataset samples` and `article datasets`.

Spec `spec/entity/dataset.md`, served by a local fixture server with `BIOMCP_PUBMED_BASE` pointed at it: one block each for `get dataset`, `dataset samples`, and `article datasets`.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Reading any file (tickets 1207 and 1211), expression values, and download (ticket 1208).
- Treatment and control labeling, parsing characteristics into typed fields, and any grouping or scoring of samples.
- `cell-line datasets`, SOFT and MINiML formats, and GSM or GPL records as their own entities.
- A new MCP tool. The generic `get` tool and the raw helper path reach the new surfaces.

## Decisions

Open to Ian's overturn: samples are a paged helper rather than a `get` section, because no `get` section takes `--offset` today.

Open to Ian's overturn: `data_as_of` means when BioMCP fetched the data, and `data_as_of_kind` has exactly two values, `release` and `retrieved`. GEO's own dates never fill it. This ticket defines both once and every other dataset ticket follows.

Open to Ian's overturn: the card defines `submitted`, `last_updated`, `status`, and `is_public` but never fetches them itself. Ticket 1211 depends on this ticket, so this ticket cannot call the header reader. A card that read a matrix file would also break the rule that `all` makes no file request. The fields render as `unknown` with a note until `series` or `assets` runs in the same invocation.

## Review

- Design review: pending
- Code review: pending
