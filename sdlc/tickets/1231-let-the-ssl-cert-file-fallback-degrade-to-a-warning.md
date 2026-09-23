# Let the SSL_CERT_FILE fallback degrade to a warning

Filed from `sdlc/issues/2026-09-23-ssl-cert-file-fallback-fails-closed-for-users-who-never-opted-in.md`. Blocks 0.9.1. Reverses part of ticket 1221's recorded fail-closed stance.

## Problem

Ticket 1221 made an unparseable CA bundle fail every command with the
`ca_bundle` error. That is right for `BIOMCP_CA_BUNDLE`, which the operator
sets deliberately. `SSL_CERT_FILE` is ambient: a system-wide variable the user
never set for BioMCP. A readable `SSL_CERT_FILE` that fails to parse now
breaks every command on upgrade, with confirmed inputs: an empty file, one
bad certificate among good ones, a leading byte-order mark, an OpenSSL
`BEGIN TRUSTED CERTIFICATE` file (the format of Fedora's
`ca-bundle.trust.crt`), and a DER file.

## Design

1. Split the failure policy by origin in `src/sources/ca_bundle.rs`
   (`:107`, `:124`): an explicit `BIOMCP_CA_BUNDLE` still fails closed with
   the path named; any problem with the `SSL_CERT_FILE` fallback warns on
   stderr and continues with the bundled roots, exactly as an unreadable
   fallback already does.
2. Extend the TLS contract tests so each confirmed bad input runs under both
   variables: explicit fails closed with the `ca_bundle` code and the path;
   fallback succeeds with a warning. The tests use the subprocess reentry
   pattern (set the variable per child process, since resolution is cached
   per process).
3. Update the precedence paragraph in `docs/reference/configuration.md`.
4. Append the Resolved section to the issue file and note the policy
   reversal in this ticket's Review section.

## Acceptance

- The five confirmed inputs behave as designed under both variables, shown
  by the new tests on yellow.
- Full yellow gate at the head SHA: `make lint`, `make test`, `make spec`.
- A record lands in `sdlc/records/`; the issue file gains a Resolved section.

## Review

- Design review: pending
- Code review: pending
