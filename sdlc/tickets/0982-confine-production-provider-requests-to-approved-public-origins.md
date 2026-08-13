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

For production defaults, ordinary HTTP provider requests must use HTTPS,
resolve only to public destinations, and remain on the initial request origin
across redirects. Origin means the exact scheme, host, and effective port.
Explicit operator base-URL overrides are process-level trusted origins: their
exact HTTP or HTTPS origin may resolve to loopback, private, link-local, or
on-prem addresses so deterministic fixtures and private deployments remain
possible. The exception does not trust a different scheme, host, or port, and
redirects always remain on the initial origin.

BioMCP uses direct provider connections. Every covered client disables ambient
`HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, and lowercase equivalents rather than
letting a proxy perform DNS outside this boundary. A provider credential may be
sent to that provider's exact configured base origin, but same-origin redirect
enforcement must prevent it from reaching any other origin.

The structural inventory covers the cached and uncached shared Reqwest clients,
the streaming client, VAERS, g:Profiler, CSpec, cBioPortal DataHub archives, and
the source health paths which construct those clients. It excludes test-only
clients and provider-returned URL clients that already install the stronger
`ProviderUrlPolicy`. AlphaGenome is a separate authenticated gRPC/Tonic
transport: it does not use Reqwest or ambient HTTP proxy variables and remains
outside this ticket's ordinary HTTP boundary. Reuse one policy owner rather
than adding client-specific checks.

## Done when

- Every ordinary provider client and the cBioPortal archive client is covered
  by a structural inventory of outbound Reqwest construction; a new unowned
  production `reqwest::Client` fails the inventory test.
- Production-default requests reject private, loopback, link-local, metadata,
  non-HTTPS, and unreviewed redirect destinations at every hop.
- An explicit base-URL override remains usable for deterministic local tests
  and on-prem operation only at its exact origin.
- Connector-level tests cover rejected private DNS, a working exact local
  override, same-origin and cross-origin redirects, ignored ambient proxies,
  and credential confinement without contacting public providers.

## Authorized test changes

Design may restate shared client and redirect assertions in
`src/sources/rate_limit.rs`, `src/sources/provider_url_policy.rs`, and
`src/sources/cbioportal_download.rs`, along with focused source-client tests
that already own base-URL overrides and local fixture servers.
