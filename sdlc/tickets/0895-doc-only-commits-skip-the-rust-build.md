---
flow: quickfix
priority: 2
---
# Doc-only commits skip the Rust build

Committing a planning note or ticket file to this repo runs the full
pre-commit pipeline, which compiles the Rust workspace — about 25
seconds per commit even when no source file changed. The operator
commits notes and tickets many times a day; on 2026-08-09 that was
well over five minutes of compilation for zero code changed.

## Done when

A commit whose staged files are all documentation — markdown under
`sdlc/`, `docs/`, or top-level `*.md` — completes pre-commit without
invoking cargo. Any staged file outside that set runs the full
pipeline exactly as today. The credential scan still runs on every
commit, including doc-only ones: notes can leak secrets as easily as
code.
