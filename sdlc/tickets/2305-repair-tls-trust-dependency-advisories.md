---
priority: 1
---

# 2305 — Repair TLS trust dependency advisories

Status: OPEN.

Milestone: 0.9.4

## Outcome

The locked TLS dependency graph no longer contains the certificate-trust versions rejected by RUSTSEC-2026-0382 and RUSTSEC-2026-0381. Preserve BioMCP's certificate verification, configured trust roots and existing transport behavior. Update the smallest compatible dependency set, without advisory suppression or unrelated upgrades.

Changes: `Cargo.lock`, `Cargo.toml`, `CHANGELOG.md`, `sdlc/records/2305-tls-trust-dependency-advisories.md`.

Proof: The unchanged lockfile fails the current advisory check; the repaired graph passes the same check and existing TLS/transport controls, mechanical checks and routine suite on branch plus current main. A fresh review precedes landing; the actual repaired main passes its full check before ordinary landings resume.

## Reproduction before code

Main is91118b5881ccbf4186875e445173b61ec4dbc86d. Its full job2298/1792523527125-59861 passed before the later advisory rejection. The record-only2284 candidate has an identical Cargo.lock and failed make landing-gate at cargo deny check advisories. Retained gate tree65e14ef3ea7b387774f84985aebb5cec2006c1da reports rustls-webpki0.103.15 under RUSTSEC-2026-0382 and webpki-roots1.0.6 under RUSTSEC-2026-0381. This new failure blocks the waiting administrative landing; it does not reclassify its earlier full check as a pass under the new feed.

The log gives primary advisory URLs https://rustsec.org/advisories/RUSTSEC-2026-0382 and https://rustsec.org/advisories/RUSTSEC-2026-0381, plus upstream rustls/webpki pull546 and rustls/webpki-roots pull133. Verify their patched versions and compatibility before resolution. Preserve the original log and exact advisory database identity when available. Do not invent an exploit result or downgrade the severity from application reachability assumptions.

## Order

This is the sole dependency repair. The dedicated lander owns it and then the waiting2284 text landing; other reliability lanes continue development without touching Cargo.lock or suppressing advisories.
