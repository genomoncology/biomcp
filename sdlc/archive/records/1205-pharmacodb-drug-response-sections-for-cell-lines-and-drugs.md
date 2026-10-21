---
flow: build
priority: 4
deps: ["1202"]
---

# PharmacoDB drug response sections for cell lines and drugs

`biomcp get cell-line <accession> drug_response` and `biomcp get drug <name> cell_lines` print the PharmacoDB experiment count per dataset and list no rows. Rows come from three helpers that each require a filter: `drug cell-lines <name> --cell-line <id>`, `drug cell-lines <name> --dataset <name>`, and `cell-line drug-response <accession> --dataset <name>`. The cell-line join takes the PharmacoDB cross-reference from the Cellosaurus record, falls back to a name lookup, and accepts a record only when its `accession_id` equals the requested accession. Experiment bodies read through a 32 MiB cap, above the largest measured input. Every output carries the retrieval time, because PharmacoDB publishes no version, and one fixed attribution line that states what the provider does and does not publish.

## Evidence

The work landed on `origin/main` and gated green on the build host at commit `3b1d6452`. `make lint` passed. `make test` passed with 3,755 of 3,755 Rust tests and 952 Python contracts passing. `make spec` passed across 30 pages. Recorded fixtures under `testdata/sources/pharmacodb/` answer every test, and synthetic bodies cover the 78,373-row K-562 counts case, the 19.4 MB full-field case, and a body over the cap. The spec fixture keys PharmacoDB cell lines by the UID Cellosaurus carries, `MOLM13_950_2019` for MOLM-13.

The licence claim the ticket opened with does not hold and was corrected in place. PharmacoDB publishes no licence and no terms page: every path on `pharmacodb.ca` returns the same 2,258-byte single-page shell, and the application bundle carries no licence string. The source code is GPL-3.0 and the CC BY-NC 4.0 belongs to the describing NAR paper, not the database. Both registries record tier 3 with a null terms URL, and a test pins that the attribution line and the two registries agree.

## Boundary

Sections print counts only, because a whole-drug or whole-cell-line listing runs past 18 MB for common inputs. BioMCP never ranks, thresholds, aggregates across datasets, or labels a line sensitive or resistant, and it merges no repeated experiment. The ticket added no tissue filter, no dose-response curve, no biomarker association, and no new MCP tool. DepMap, PRISM downloads, GDSC direct access, and LINCS stay out; PharmacoDB is BioMCP's route to GDSC1 and GDSC2.
