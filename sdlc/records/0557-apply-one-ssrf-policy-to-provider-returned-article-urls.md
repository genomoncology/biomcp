---
base: 9279a6c8964ba46764a4fd6db0635de3e53c9d0f
head: 6ab51fbc3ee4f048b8e8024ac1559c127cd71475
---
BioMCP fetches provider-returned URLs and follows their redirects. A compromised or malicious provider payload can steer a fetch toward a private, link-local, loopback, cloud-metadata, non-HTTPS, or off-origin target — a server-side request forgery (SSRF) sink. Compounding this, `SemanticScholarClient` attaches the real `x-api-key` whenever `BIOMCP_S2_BASE` is overridden and the key is nonempty, so a base override — or a redirect to a noncanonical origin — leaks the operator credential to an unapproved destination. URL destination and credential attachment are the same outbound-request trust boundary, so one policy must own both.

Imported from March ticket 557. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/557-apply-one-ssrf-policy-to-provider-returned-article-urls
