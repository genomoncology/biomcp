---
base: 7a21702f
head: a71b0e98
---

Finished the label warnings work from the issue filed 2026-09-23.

The safety-path warnings now truncate at the same 2,000-character cap as
the boxed warning beside them, and the truncation note carries the exact
DailyMed URL built from `label_set_id`, independent of `drug.label`
being present. Both renderers use one `### Boxed Warning` heading,
legacy text renders under a neutral `### Warnings` heading, and the
Markdown renderer suppresses the safety copy of the boxed warning only
when the visible label section already rendered it — deduplication
lives in rendering (`src/render/markdown/drug.rs`), with both model
fields independent for separate sections and JSON. The two render-order
tests moved to a new `label_warnings.rs` module; `drug/tests.rs`
returned under its floor and lost its size-inventory entry, and the
retained provenance entry's reason now names the three fixtures that
initialize the field. Docs mention boxed warnings in the user guide and
the openFDA source page.

Named regressions: one boxed heading when `label` and `safety` are
combined; a truncated safety warning containing the exact set-id
DailyMed URL when `drug.label` is absent; plus precedence (both
`warnings_and_cautions` and `warnings`), a JSON `boxed_warning`
assertion, and multi-string and truncated boxed coverage.

Evidence: code review ACCEPT with no P2s (Sol medium); yellow gate at
a71b0e98 — `make lint` OK, `make test` OK, `make spec` OK. The first
gate run failed only the GenCC lease flake (third occurrence, filed as
ticket 1239) and the `src/render/json.rs` size baseline, raised to 1,570
with a ticket-1237 authorization in the same commit.

Residuals: none specific to this ticket; the GenCC flake tracks in
1239.
