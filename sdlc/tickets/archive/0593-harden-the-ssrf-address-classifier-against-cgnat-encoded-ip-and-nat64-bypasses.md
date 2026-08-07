---
flow: build
priority: 5
---
# Harden the SSRF address classifier against CGNAT, encoded-IP, and NAT64 bypasses

The SSRF outbound-URL policy (tickets 557/584) is strong — DNS-layer enforcement, per-hop redirect re-validation, and an origin allowlist backstop — but an adversarial audit (2026-07-18) found `is_forbidden_address` in `src/sources/provider_url_policy.rs` misses a few exotic internal-address encodings:

Completed under March on 2026-07-19, as March ticket 593. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/593-harden-the-ssrf-address-classifier-against-cgnat-encoded-ip-and-nat64-bypasses
