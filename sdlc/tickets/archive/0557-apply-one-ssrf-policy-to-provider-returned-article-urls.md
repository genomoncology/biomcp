---
flow: quickfix
priority: 11
---
# Apply one SSRF policy to provider-returned article URLs

BioMCP fetches provider-returned URLs and follows their redirects. A compromised or malicious provider payload can steer a fetch toward a private, link-local, loopback, cloud-metadata, non-HTTPS, or off-origin target — a server-side request forgery (SSRF) sink. Compounding this, `SemanticScholarClient` attaches the real `x-api-key` whenever `BIOMCP_S2_BASE` is overridden and the key is nonempty, so a base override — or a redirect to a noncanonical origin — leaks the operator credential to an unapproved destination. URL destination and credential attachment are the same outbound-request trust boundary, so one policy must own both.

Completed under March on 2026-07-15, as March ticket 557. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/557-apply-one-ssrf-policy-to-provider-returned-article-urls
