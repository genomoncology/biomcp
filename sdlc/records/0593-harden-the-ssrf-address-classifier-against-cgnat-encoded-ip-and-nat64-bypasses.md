---
base: eee9b1b4885269d63cd1f7b8889ca7d16cf13d6c
head: 73f4c183aa67777847b199206e95632dfb9a9bf2
---
The SSRF outbound-URL policy (tickets 557/584) is strong — DNS-layer enforcement, per-hop redirect re-validation, and an origin allowlist backstop — but an adversarial audit (2026-07-18) found `is_forbidden_address` in `src/sources/provider_url_policy.rs` misses a few exotic internal-address encodings:

Imported from March ticket 593. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/593-harden-the-ssrf-address-classifier-against-cgnat-encoded-ip-and-nat64-bypasses
