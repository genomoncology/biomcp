# Bind article search Markdown fields to identifiers

File/line: `src/render/markdown/article/tests.rs:577-587`
Severity: should-fix

The ranked-search test checks article identifiers for order but searches titles,
sources, and rationale globally. Swapping those fields between rows still
passes, so a result can show the right rank with another paper's metadata.
Assert each complete result row, including its identifier, title, sources, and
ranking rationale, before retaining the rank-order assertion.
