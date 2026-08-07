---
flow: quickfix
priority: 10
---
# Extend the outbound-URL policy to remaining provider consumers

Ticket 557 builds one shared outbound-URL SSRF policy and applies it end-to-end to the Semantic Scholar client (the path that carries the `x-api-key` credential and the highest-risk PDF-fallback sink). The remaining consumers that fetch provider-returned URLs — PMC OA manifest retrieval and the Figshare/trial document paths — still reach the network without that policy, so each is an independent server-side request forgery (SSRF) sink that a compromised provider payload can steer toward loopback, private, link-local, cloud-metadata, non-HTTPS, or off-origin targets. Nothing structurally prevents a *new* fetch consumer from being added that bypasses the policy silently. This ticket adopts 557's policy across those consumers and adds an enumerating ratchet so every provider-returned-URL fetch site is forced through one owner.

Completed under March on 2026-07-16, as March ticket 584. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/584-extend-the-outbound-url-policy-to-remaining-provider-consumers
