---
priority: 2
---

# 2240 — Use the CA bundle for AlphaGenome

Status: OPEN.

Milestone: 0.9.4

## Outcome

`get variant ... predict` uses `BIOMCP_CA_BUNDLE` like every other source, so it works behind a proxy that re-signs TLS.

## Evidence

`sdlc/issues/2026-09-23-ca-bundle-follow-ups.md`; deferred by 1236 and documented as a limitation in `docs/reference/configuration.md` near line 70. The AlphaGenome gRPC client builds its TLS config without the loaded bundle.

## Keeps

Without the setting, behavior is unchanged. Remove the documented limitation once fixed.

## Proof

A test against a local TLS server signed by a test CA fails without the bundle and connects with it. Mark the issue finding resolved.
