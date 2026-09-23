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
   fallback succeeds with a warning. The existing contract tests already
   spawn the real binary per case with the bundle variable set per child
   (`tests/tls_ca_bundle_contract.rs:59-77`); the `run()` helper needs only
   an `SSL_CERT_FILE` variant. Add one `--json` explicit-variable case (the
   JSON path is proven at `:249`).
3. Update the precedence paragraph in `docs/reference/configuration.md:59-66`:
   the sentence at `:61-62` ("a readable but malformed bundle fails") is the
   contradiction. State plainly that a fallback bundle that cannot be parsed
   is dropped whole, good certificates included, after one warning.
4. Append the Resolved section to the issue file and note the policy
   reversal in this ticket's Review section. The health runner
   (`src/cli/health/runner.rs:296-304`) picks up the new policy through the
   same `ca_bundle::build_client` path with no code change; record that.

## Acceptance

- The five confirmed inputs behave as designed under both variables, shown
  by the new tests on yellow.
- Full yellow gate at the head SHA: `make lint`, `make test`, `make spec`.
- A record lands in `sdlc/records/`; the issue file gains a Resolved section.

## Review

- Design review: ACCEPT 2026-09-23 (gpt-5.6-sol, medium) — single-seam finding
  and P2 notes folded above
- Code review: ACCEPT 2026-09-23 (gpt-5.6-sol, medium) — two test-strength
  findings (per-case connection delta, degrade warning asserted on stderr)
  applied in 63678004 and 5e712966
- Verification: yellow gate at 5e712966 lint/test/spec OK; TLS contract
  suite 8/8 three times; see
  `sdlc/records/1231-let-the-ssl-cert-file-fallback-degrade-to-a-warning.md`
