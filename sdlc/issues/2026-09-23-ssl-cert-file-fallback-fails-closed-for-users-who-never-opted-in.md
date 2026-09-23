# SSL_CERT_FILE fallback fails closed for users who never opted in

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`. Blocks 0.9.1. Regression from ticket 1221.

## Symptom

0.9.0 ignored `SSL_CERT_FILE`. Now a readable `SSL_CERT_FILE` that fails to parse makes every command fail with `ca_bundle` (`src/sources/ca_bundle.rs:107`, `:124`). Confirmed failing inputs:

- an empty file
- one bad certificate among good ones
- a leading byte-order mark
- an OpenSSL `BEGIN TRUSTED CERTIFICATE` file (the format of Fedora's `ca-bundle.trust.crt`)
- a DER file

The stock Ubuntu bundle loads fine. A user with a system-wide `SSL_CERT_FILE` in one of these formats upgrades and loses every command without having set any BioMCP variable.

## Fix

A problem with the `SSL_CERT_FILE` fallback warns and continues, as a missing file already does. Only `BIOMCP_CA_BUNDLE` fails closed. Add a subprocess test for each input above under both variables.

## Resolved

Resolved by ticket 1231
(`sdlc/tickets/1231-let-the-ssl-cert-file-fallback-degrade-to-a-warning.md`)
on branch `tickets/1231-ca-fallback-warn`: a parse failure in the
`SSL_CERT_FILE` fallback now warns and continues with the bundled roots,
while `BIOMCP_CA_BUNDLE` keeps failing closed with the path named.
