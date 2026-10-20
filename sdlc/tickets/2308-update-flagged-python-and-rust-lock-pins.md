---
priority: 1
---

# 2308 — Update flagged Python and Rust lock pins

Status: OPEN.

Milestone: 0.9.4

## Outcome

The locked versions clear every open GitHub security alert that a patch-level or minor update fixes: certifi 2026.1.4 to 2026.10.19 or later (critical; distrusts the compromised emSign roots, the same issue 2305 fixed in rustls), urllib3 2.7.0 to 2.8.0 or later (two high, one moderate), mkdocs-material 9.7.2 to 9.7.7 or later (moderate, docs search XSS) and xxhash-rust 0.8.15 to 0.8.16 or later (low). The changelog names the security updates.

- Proof: `uv.lock` and `Cargo.lock` carry the patched versions with the smallest change; the routine suite, docs build and installed-wheel smoke pass; after landing, GitHub shows alerts 80, 87, 88, 89, 91 and 93 closed.

## Evidence

GitHub Dependabot listed these on 2026-10-20 (alerts 80, 87, 88, 89, 91, 93). The local `cargo deny` gate did not report xxhash-rust, and no local check audits `uv.lock`, so the routine gate never saw them.

Changes: uv.lock, Cargo.lock, CHANGELOG.md
