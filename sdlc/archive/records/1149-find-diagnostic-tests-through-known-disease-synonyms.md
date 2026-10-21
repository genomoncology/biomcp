---
flow: build
priority: 6
deps: []
---

# Find diagnostic tests through known disease synonyms

BioMCP diagnostic search now resolves one exact MyDisease identity for GTR
disease filters and searches the caller's term, canonical disease name, and up
to twenty exact provider synonyms. It makes at most two logical MyDisease
requests, fails safely on incomplete or inconsistent provider data, and never
broadens through fuzzy matching, ontology traversal, crosswalks, or discover.
WHO IVD remains literal-only and neither WHO-only nor non-disease searches call
MyDisease.

Each matching row optionally reports whether its condition matched the
requested term, canonical name, or a synonym, including the resolved ontology
ID when available. Requested matches rank before canonical matches, which rank
before synonyms in provider order across all conditions. The ranking happens
before offset and limit, while other filters remain conjunctive and existing
cross-source total semantics are unchanged. JSON, Markdown, and raw MCP share
the result, and raw MCP preserves CLI bytes when no local path redaction is
required.

## Evidence

A fresh SOL design review rejected ambiguity around incomplete provider pages,
cross-source ranking, provider term shapes, and model routing. The corrected
focused design was accepted before implementation. Luna produced the initial
implementation; SOL completed hardening after focused tests exposed fixture
ownership and MCP byte-equivalence gaps.

Independent SOL code review then rejected two defects: condition order could
override requested/canonical/synonym precedence, and unchanged-byte handling
could preserve a null or malformed `full_text_path`. The remediation applies
term priority across every row condition and reserializes whenever any nested
`full_text_path` key is removed. Regression tests cover cross-condition
priority, sort-before-pagination, nested/null/non-string/duplicate path keys,
and unchanged bytes only when the key is absent. The same reviewer accepted
the exact remediated revision with no remaining material finding.

On that accepted revision, `make lint`, `make test`, `make spec`, and `make
full-feature-check` passed under Node 22. The shared host used eight nextest
workers: all 3,415 Rust tests passed; 955 Python contracts passed with three
intentional skips; strict documentation and every offline executable/static
specification group passed. All-feature Clippy, the optimized all-feature
build, six AlphaGenome behavior tests, and PNG/SVG/terminal artifact smoke
passed. `cargo package --list --allow-dirty --locked --offline --no-verify`
reports exactly 1,300 paths, and `git diff --check` passes.
