---
flow: build
priority: 6
---
# Confine production provider requests to approved public origins

Provider-returned download URLs already use BioMCP's strong outbound policy,
but ordinary API clients use Reqwest's default redirect and DNS behavior. A
compromised upstream or poisoned DNS answer can redirect a server-side BioMCP
request to loopback, a private network, link-local infrastructure, or cloud
metadata. Fixed upstream URLs reduce likelihood but do not enforce the boundary.

For production defaults, provider requests must use HTTPS, resolve only to
public destinations, and remain on reviewed origins across redirects. Explicit
operator base-URL overrides are trusted so local fixtures and deliberate
on-prem deployments remain possible, but redirects from an override stay on
that configured origin unless a documented operator setting allows another
one. Reuse one policy owner rather than adding client-specific checks.

## Done when

- Every ordinary provider client and the cBioPortal archive client is covered
  by a structural inventory of outbound request construction.
- Production-default requests reject private, loopback, link-local, metadata,
  non-HTTPS, and unreviewed redirect destinations at every hop.
- An explicit base-URL override remains usable for deterministic local tests
  and on-prem operation, while its redirect scope follows the rule above.
- Tests cover DNS answers and redirect chains without contacting public
  providers, and credentials are attached only to their approved origin.

## Authorized test changes

Design may restate shared client and redirect assertions in
`src/sources/rate_limit.rs`, `src/sources/provider_url_policy.rs`, and
`src/sources/cbioportal_download.rs`, along with focused source-client tests
that already own base-URL overrides and local fixture servers.
