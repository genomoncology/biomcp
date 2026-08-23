# Bind disease Markdown test rows to their sources

File/line: `src/render/markdown/disease/tests/rendering.rs:266-287`
Severity: should-fix

The evidence-link and clinical-feature tests search the whole Markdown for
cells independently. They pass if sources move between gene, phenotype, and
model rows, or if clinical-feature columns are reordered or split. Assert each
complete row within its owning section, and update the disease spec's table
assertions to select that section and row.
