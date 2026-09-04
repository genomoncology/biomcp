# One ClinGen failure erases other completed results

Severity: should-fix

Two no-cache requests for `biomcp --json get gene TP53 clingen` returned an empty `clingen` object and an unavailable section on 2026-09-04. Debug logging reported `ClinGen gene section timed out` after eight seconds. Direct requests to ClinGen's gene lookup, gene-validity download, and gene-dosage download all responded when tested separately.

`ClinGenClient::gene_context` in `src/sources/clingen.rs` runs the lookup and two downloads concurrently. It then uses `?` on each bulk-download result. One failed request discards any completed result from the other source. `fetch_clingen_section` in `src/entities/gene.rs` places one timeout around the combined operation. One slow request therefore turns all ClinGen evidence into one unavailable section.

TP53 supplies a useful regression case because the public datasets contain established records. A deterministic test can delay or fail one source and prove that BioMCP retains the other result. Each result family also needs its own outcome. An empty matched set, an unavailable source, and a completed source with records have different meanings.

## Success criteria

- A delayed dosage response does not erase completed TP53 gene-validity records.
- A failed validity response does not erase completed TP53 dosage records.
- Output reports status separately for gene validity and dosage sensitivity.
- A cold request receives a fair chance to establish the public downloads without hiding which operation timed out.
- Fixed provider fixtures prove partial success without live requests.

Found on 2026-09-04 during a rare disease case research exercise.
