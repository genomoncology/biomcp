---
flow: build
priority: 5
deps: ["1202"]
---

# ChEMBL cell line section

`biomcp get cell-line <accession> chembl` prints the ChEMBL cell line records that name the requested Cellosaurus accession, each with its ChEMBL ID, EFO ID, CLO ID, and assay count. The join goes only through `cellosaurus_id`, because ChEMBL spells four of the ten AML test lines differently from Cellosaurus. The section reads the release name and date from ChEMBL `status.json` at runtime and falls back to the retrieval time when that call fails. `get cell-line <accession> all` leaves the section out, so it costs nothing unless it is asked for.

## Evidence

The work landed on `origin/main` and gated green on the build host at commit `fd6a100c`. `make lint` passed. `make test` passed with 3,676 of 3,676 Rust tests and 952 Python contracts passing. `make spec` passed across 30 pages. Recorded ChEMBL fixtures under `testdata/sources/chembl/` cover the MOLM-13 record with null EFO and CLO IDs, the K-562 record CHEMBL3308378 with EFO_0002067 and CLO_0007059, an empty result, the assay count page, and `status.json`.

## Boundary

The section lists no assays, no activity values, and no cell-line targets. It never searches ChEMBL by name, and it adds no ChEMBL-to-Cellosaurus reverse lookup, because ticket 1202 already resolves ChEMBL IDs through the Cellosaurus `dr:` field.
