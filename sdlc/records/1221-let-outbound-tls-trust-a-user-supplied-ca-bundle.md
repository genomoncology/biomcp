---
base: c6da3ef7
head: 70584666
---

Let outbound HTTPS trust an operator-supplied CA bundle.

The shared reqwest clients used `rustls-tls`, which is the bundled webpki
roots only, so networks that route through an internal root CA could not reach
any provider. A new `src/sources/ca_bundle.rs` reads `BIOMCP_CA_BUNDLE`, falls
back to `SSL_CERT_FILE` only when it is unset, parses every PEM certificate,
validates each one's DER by adding it to a local rustls root store, requires at
least one certificate, and adds the parsed certificates to the client builder.
The roots are additive: nothing replaces the bundled set and no path disables
verification. A missing, unreadable, malformed, or certificate-less bundle
fails before any request with a `ca_bundle` error that names the path in text
and `--json`. Blank or whitespace-only `BIOMCP_CA_BUNDLE` counts as unset; a
blank or unreadable `SSL_CERT_FILE` warns and continues, while a readable but
malformed one fails naming the path.

The helper is wired into the three ordinary builders, the middleware clients,
orcid, ClinGen CSPEC, both fda_orphan clients, trial document downloads, and
the health client. `fda_orphan::fetch*` now returns a `Result` whose only `Err`
is the CA-bundle configuration error, so the optional lane still degrades to
`unavailable()` for every other failure. AlphaGenome's gRPC client already
reads the native OS trust store and was left alone. `ca_bundle` is classified
as a source-registry helper module, and the package-file count rose to 1,346
for the new helper and its contract test.

Evidence at 70584666 on yellow: `cargo metadata --locked` consistent, `cargo
fmt --check` and `cargo clippy --locked --all-targets -- -D warnings` clean,
the five-case TLS contract test (configured success, missing, malformed,
certificate-less, unreadable, plus JSON path naming) passes three times, the
full nextest run passes 3776/3776, `make lint` passes, and the package-boundary
lane passes 9/9 with `TMPDIR` inside the worktree. 85 focused Python docs and
policy tests pass locally. The live docs pointer and CI `canonical-gates` were
green on the merged main before this branch. The first `canonical-gates` run
after the merge failed on the licensing inventory test, which needed
`ca_bundle` added to its helper-module exclusion set; that fix landed on main
as `91c0322a`.

Reviews: the design review rejected the first draft and required the full
client-coverage list, in-process DER validation, defined precedence, and a
concrete TLS fixture plan. The code review accepted with six P2 notes; the
delta review closed all six, including the `var_os` read, dropping the
unmaintained `rustls-pemfile`, the unreadable-bundle test, the registry
classification, and the package count.

Residual: no test covers a non-UTF-8 `BIOMCP_CA_BUNDLE` value or a multi-
certificate bundle with one invalid certificate; the unreadable-bundle case
passes silently on a privileged runner; and the fda_orphan lane builds its
client per call, so a mid-process environment change affects that lane even
though the shared and health clients are built once. Issue #250 stays open
until 0.9.1 ships.
