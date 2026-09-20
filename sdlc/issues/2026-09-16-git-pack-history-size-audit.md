# Git pack is 1.23 GiB against a roughly 30 MB working tree

Observed 2026-09-16 during the v0.8.25..v0.9.0 growth audit.

`git count-objects -vH` reports size-pack 1.23 GiB, loose size 45.54 MiB, and 26 packs. The directories that matter to a build (`src/`, `tests/`, `spec/`, `testdata/`, `benchmarks/`) total roughly 30 MB on disk. History carries nearly all of the repository's weight, and every full clone pays it.

This ticket does not assert what the weight is. That needs an audit: `git filter-repo --analyze` or a `git verify-pack` top-blob listing to name the largest historical blobs and the commits that churned them. Candidates include deleted build output, retired fixtures, and data files replaced across releases. Confirm by audit, not assumption.

Remediation is a decision for Ian, not a default. A history rewrite changes every SHA. That touches tags, the Homebrew formula, PyPI provenance, and both machines' clones. Partial clone with a blob filter shrinks CI and developer fetches without touching history.

Worth considering: run the audit first, then choose between rewrite, partial-clone policy, and acceptance with a check that blocks new large blobs at the gate.
