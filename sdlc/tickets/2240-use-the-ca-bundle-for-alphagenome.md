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

## Focused proof

Source 795c5e01fcfe7c4bbe324c2c4575126068a2f641 passes the private-CA connection regression after the baseline failed. The first parallel TLS run exposed three existing startup deadlines; review fixed those with one shared watchdog in 2e7f610c130e29eda3020e0920558afc3e4c1733 while preserving behavior assertions. Review job 1792421520545-14590 passed all twenty TLS cases in parallel, focused all-feature Clippy and make lint. Earlier proof also passed six shared-loader cases, six Python policy checks and six AlphaGenome library cases; published correspondence matched all nine files. No live prediction RPC is claimed. Private experiment 780-biomcp-2240-alphagenome-ca retains raw attempts; reviewer-parallel.log SHA256 is 1fc8a1997d1e919da5a4c99d540b28e29d17c8104d494e6c447afbcc0a098f1d.
