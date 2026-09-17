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
- No code calls `elink` today (`grep -rn elink src` finds nothing). The PubMed client wraps only `esearch` and `esummary` for PubMed (`src/sources/pubmed.rs:338`, `src/sources/pubmed.rs:410`) against `BIOMCP_PUBMED_BASE` (`src/sources/pubmed.rs:12-14`). The generic E-utilities helper from 1203 gains `elink` here.
- Entities parse named sections with a local `parse_sections` (`src/entities/pathway.rs:121`). Article helpers live in `ArticleCommand` (`src/cli/article/mod.rs:308`). `article entities <pmid>` (`:328`) is the model for a PMID-keyed pivot with `--limit`.
- `all` is the default surface, not every opt-in section (`architecture/ux/cli-reference.md:97-104`).
- The source registry already has an `ncbi-e-utilities` row (`docs/reference/sources.json:979`). 1203 adds the GEO source page.

GEO facts the design relies on, to confirm against the recorded fixtures:

- `esummary` with `db=gds` returns, per series, `accession`, `entrytype`, `gpl`, `n_samples`, `bioproject`, `relations`, and a `samples` array of `{accession, title}`.
- Experiment 203 found 5,912 sample IDs in more than one of 585 series (774 files). A SuperSeries repeats the samples of its SubSeries. GSE982 is a SubSeries of GSE995.
- In the same survey, sample characteristics and sample titles were the first field that split samples by a treatment word in 569 of 623 files. The characteristics keys that split samples most often were `treatment` (336 series) and `agent` (41). The treatment protocol mentioned DMSO in 328 series and was usually the same for every sample.
- `elink` with `dbfrom=pubmed&db=gds` returns GDS UIDs of mixed record types (series, platforms, samples, curated DataSets).

## Design

### `get dataset <id>` card

`get dataset <id>` accepts `geo:GSE…` or bare `GSE…` and renders the 1203 row for that series from one `esummary` call: ID, title, organisms, series types, platforms, sample count, PMIDs, supplementary types, `geo2r`, and summary. Sections `publications` (the PMIDs with `get article` next commands) and `links` (the GEO page, BioProject, and SuperSeries/SubSeries relations) read the same response. The card names the SuperSeries or SubSeries relation when one exists.

`all` means the sections that make no file request: the card, `publications`, and `links`. It never includes `series` (ticket 1211) or `assets` and `products` (ticket 1207).

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
- Add the sections, the helper, and the pivot to `biomcp list dataset` and `biomcp list article`.
- State that a SuperSeries repeats its SubSeries samples. Any tally across series counts distinct GSM IDs.
- State that sample maps for ticket 1209 usually come from characteristics keys such as `treatment` and `agent` and from sample titles. The treatment protocol rarely tells samples apart.

## Acceptance

Fixtures under `testdata/sources/geo/`, small and recorded: `esummary` JSON for one single-platform series with three samples and one two-platform series with relations, and `elink` JSON for one PMID linking to a series, a platform, and a sample, plus one PMID with no links.

1. `get dataset GSE982` and `get dataset geo:GSE982` render the same card.
2. `publications` and `links` render from the fixture with no extra request. The two-platform fixture shows its SuperSeries or SubSeries relation on the card and in `links`.
3. `dataset samples` renders samples in provider order and pages with `--offset`.
4. `article datasets` keeps only `GSE` rows from the mixed `elink` fixture and returns the empty note for the unlinked PMID.
5. `get dataset <GSE>` and `get dataset <GSE> all` send only the `esummary` request.

Spec `spec/entity/dataset.md`, served by a local fixture server with `BIOMCP_PUBMED_BASE` pointed at it: one block each for `get dataset`, `dataset samples`, and `article datasets`.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Out of scope

- Reading any file (tickets 1207 and 1211), expression values, and download (ticket 1208).
- Treatment and control labeling, parsing characteristics into typed fields, and any grouping or scoring of samples.
- `cell-line datasets`, SOFT and MINiML formats, and GSM or GPL records as their own entities.
- A new MCP tool. The generic `get` tool and the raw helper path reach the new surfaces.

## Decisions

Open to Ian's overturn: samples are a paged helper rather than a `get` section, because no `get` section takes `--offset` today.
