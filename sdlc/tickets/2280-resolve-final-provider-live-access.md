---
priority: 1
---

# 2280 — Resolve final provider live access

Status: OPEN.

Milestone: 0.9.4

Reviews: revision beef86e, accept

## Outcome

Determine the supported, authorized path to complete live replay verification for Reactome, WikiPathways and Semantic Scholar, or identify the exact external blocker and next owner. No denial or rate-limited reply qualifies a recording.

Changes: `sdlc/records/2280-final-provider-live-access.md`, `CHANGELOG.md`.

## Evidence

Final collection 2258/1792447971557-46385 retained 47 entries in private M5 experiment 2096-biomcp-2258-live-collection. All six Reactome requests and WikiPathways returned HTTP403 plaintext; the first S2 request returned429 without Retry-After, so four later S2 requests were not attempted. No credentials, proxy inheritance, redirects or retries were used. The stable source main is 1c4753a453569d17dd66a13eb3d312ab8a5af4d4, whose full check passed 2156/1792447869262-24945.

## Keeps

Read existing raw denial bodies, safe headers, request settings, source registry/licensing, actual client headers and current official API documentation first. Distinguish an invalid request/client header, an obsolete endpoint, a provider outage/rate limit and an explicit access-policy denial. No auth bypass, impersonation, proxy, alternate-origin/IP workaround, new account, paid access, secrets or repeated calls until green. Never call KEGG.

After that evidence, at most one diagnostic request for each affected provider may run through pm job on M5, using the actual supported public client request and truthful application identity, without credentials. Respect any retry interval and elapsed provider stop. No redirect or retry loop; bounded time and body size. An explicit policy or authentication denial ends that provider's probe. If an existing supported endpoint/header is wrong, report a separate bounded source/checker defect; do not change runtime or checker code in this experiment. If support needs privileged access, cost or a ruling, report the owner and concrete options to the coordinator.

## Proof

Retain the original and diagnostic request identity, safe response metadata, body hashes, timestamps, documentation sources, differences, actual result and limits in one private numbered experiment with OWNER and Keep. Return a direct report; coordinator records the safe public conclusion. No full gate, complete live qualifier or release claim. No test build is needed beyond the bounded configured-machine probe.

## Result and authority

The PM explicitly approved the two released-product commands and one later public Semantic Scholar probe after the initial read-only diagnosis. Successful job 2280/1792448958050-39962 establishes product access to one Reactome and one WikiPathways lookup; endpoint and transport limits remain explicit in the record. Semantic Scholar remained 429, with four original requests unattempted. The PM approved the “not checked live: rate limited” unverified disposition and separate 2281 checker correction with seven one-time rechecks. The retained private report and safe public record carry both preparation failures and successful diagnosis. No source/code changes or complete qualification are claimed.

Keep: experiments/2099-biomcp-2280-provider-access — retain diagnostic failures, verified released artifact and bounded provider access evidence.
