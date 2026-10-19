---
priority: 2
---

# 2240 — Use the CA bundle for AlphaGenome

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 2e7f610, accept

## Outcome

- Changes: `src/sources/ca_bundle.rs` `src/sources/alphagenome.rs` `tests/tls_ca_bundle_contract.rs` `tests/test_provider_network_policy.py` `docs/reference/configuration.md` `docs/reference/data-sources.md` `docs/troubleshooting.md` `CHANGELOG.md` `sdlc/issues/2026-09-23-ca-bundle-follow-ups.md` and this ticket.

`get variant ... predict` uses `BIOMCP_CA_BUNDLE` like every other source, so it works behind a proxy that re-signs TLS.

## Evidence

`sdlc/issues/2026-09-23-ca-bundle-follow-ups.md`; deferred by 1236 and documented as a limitation in `docs/reference/configuration.md` near line 70. The AlphaGenome gRPC client builds its TLS config without the loaded bundle.

## Keeps

Without the setting, behavior is unchanged. Remove the documented limitation once fixed.

## Proof

A test against a local TLS server signed by a test CA fails without the bundle and connects with it. Mark the issue finding resolved.

## Coordinator development brief

Pass the already validated cached operator CA bundle into AlphaGenome TLS while keeping enabled native roots and no-setting behavior. Source currently builds ClientTlsConfig with enabled roots/domain only. Use the real health --api AlphaGenome CLI path against the existing private-CA fixture with h2 ALPN; absent bundle must fail TLS and supplied bundle must complete the connection. Do not claim RPC prediction proof from a handshake. No dependency upgrade or extra MyVariant fixture is needed. Remove obsolete documentation assertions together.

All builds and tests run through the configured build job. Stage private test/source patches on a published base without pushing WIP; retain patch and per-file hashes with raw evidence, then verify the final published source matches. Push source once ready for review. The developer may resolve the named issue findings as required by the outcome, but does not edit ticket status or send mail.
