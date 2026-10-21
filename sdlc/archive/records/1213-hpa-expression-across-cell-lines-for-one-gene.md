---
flow: build
priority: 4
deps: ["1202"]
---

# HPA expression across cell lines for one gene

`biomcp gene cell-lines <symbol> --group <cancer group>` prints the Human Protein Atlas RNA level in nTPM for one gene across every HPA cell line of one cancer group. The symbol resolves through a single MyGene.info lookup. Each row carries a Cellosaurus accession when exactly one human Cellosaurus identifier matches the HPA name, and a row note when none does. The batch join queries `id:(...) AND ox:9606`, which keeps the recorded bytes at 613 KB across the five leukemia batches. BioMCP prints the values as published and adds no labels, thresholds, or sorting.

## Evidence

The work landed on `origin/main` and gated green on the build host at commit `6d8fd435`. `make lint` passed. `make test` passed with 3,693 of 3,693 Rust tests and 952 Python contracts passing. `make spec` passed across 30 pages. Measurement through the production request path corrected three numbers the ticket carried: the leukemia group holds 93 lines, the data access page lists 1,206 lines across 30 groups, and FLT3 in MOLM-13 is 166.1 nTPM. All 93 names resolve to exactly one human accession. The Human Protein Atlas terms page publishes CC BY 4.0 for the copyrightable parts of the database; the CC BY-SA 4.0 the repo carried was a third-party licence listed further down the same page. The attribution line, `docs/reference/source-licensing.md`, and `docs/reference/sources.json` now carry one value, and a test pins that they agree.

## Boundary

The helper reads one gene at a time. It offers no all-genes view for one cell line, which would need the 206 MB bulk file, no protein level, subcellular location, or single-cell data, no ranking or threshold, and no `get cell-line` section. HPA publishes no Cellosaurus link, so the join stays a guarded name join.
