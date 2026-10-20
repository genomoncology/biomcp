---
base: e4e510790ca9cc4854cceafb66a77c733a5055cb
head: 4446f4a1b3602793fdaa4d6b02f17349449bc6c9
---

Normal builds now include the committed AlphaGenome generated Rust source
directly. The package build script no longer invokes protoc, generates a
candidate, copies a fallback, or writes a tracked path. `tonic-build` and six
generator-only transitive crates left the main dependency lock.

The explicit maintainer command requires protoc 28.3, generates through a
small separately locked tool into a temporary target directory, applies the
reviewed dead-code annotation, validates the client surface, and atomically
replaces only the expected file. Its check mode emits a unified diff without
writing. One dedicated CI job runs that check; ordinary CI builds do not
install protoc. Source packages include the committed Rust and exclude the
generator inputs.

Six AlphaGenome tests passed with a fail-on-call protoc configured. No-feature
checks passed with protoc missing. The regeneration harness reproduced the
committed bytes, and full locked source-package verification passed in 74.65s
without invoking the configured generator. All 31 focused Python and
documentation contracts passed, as did full lint.
