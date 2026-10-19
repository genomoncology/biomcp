# Final provider access diagnosis

The released BioMCP 0.9.3 client successfully fetched Reactome R-HSA-5673001 and WikiPathways WP1 through its normal client. The earlier replay collector received seven HTTP 403 responses containing `error code: 1010`. These results establish a request-behavior difference. They do not prove User-Agent alone caused the denial. The WikiPathways product lookup uses a different endpoint from the recorded search, so that search remains unverified.

The single approved later Semantic Scholar request returned HTTP 429 without Retry-After after 909.815474 seconds since its original denial. Its 174-byte body matches the original hash, `65ab993d12c5c2cc9b68e6da2bb79b326930283e520363ffcdc145f88cd5a148`. Four other requests remain unattempted. The PM approved reporting this source as “not checked live: provider denied”. Neither denial nor unattempted requests count as successful verification. KEGG remains uncalled and commercially restricted.

## Artifact and job evidence

Job 2280/1792448958050-39962 ran from e60313187ba865814c284ab7ec83bc81d9829f85 on stable main 1c4753a453569d17dd66a13eb3d312ab8a5af4d4. It downloaded the official published `biomcp-darwin-arm64.tar.gz` release asset from the [0.9.3 release](https://github.com/genomoncology/biomcp/releases/tag/v0.9.3), verified its published sidecar and checked the extracted version. Archive SHA256: `e637062c18c6692f3ec49c5847e69b29f6feee7f7909345970bd504d72174994`. Binary SHA256: `54f66769d9193c569d1b65756ad766eafd1a2c5e92d08f8c855cc8964e9a7145`. Tag source: 5641de7d7b79b53872f8599dbb4dba22e611dcaf. The installed older binary was not used.

Both `get pathway R-HSA-5673001 --no-cache` and `get pathway WP1 --no-cache` exited successfully with the requested identifiers and source names, using isolated configuration and cache, no inherited credentials or proxies, and licensed sources disabled. Product stdout hashes are not provider-body hashes. Native wire status, response headers and attempt counts were not captured; the normal client's bounded transient retry policy remained intact. No added retry loop ran. The separate Semantic Scholar diagnostic disabled redirects and retries and bounded time and body size.

Two earlier preparation jobs, 1792448923200-33505 and 1792448941511-36065, failed before downloads or provider calls because of asset-name and isolated PATH assumptions. Both failures remain retained; neither made a provider retry.

Private experiment `2099-biomcp-2280-provider-access`, owned by biomcp/2280 and marked Keep, retains report.txt, original observations, documentation/source evidence, preparation failures, verified release artifacts, probe code and product-results.json. Original collection remains separately in `2096-biomcp-2258-live-collection`. Raw provider replies remain private. No credentials were inspected, stored or sent.

## Disposition

The original urllib collector used its default Python identity. The released client uses its versioned BioMCP identity, default Accept header and no-cache behavior; transport defaults also differ. Exact original wire defaults were not captured. Official documentation supports signature-denial and rate-limit interpretations, but it does not establish a particular CDN rule or mandatory authentication.

Ticket 2281 owns the PM-approved checker header correction and one bounded recheck of the seven denied requests. No further Semantic Scholar requests are authorized. Any continued denial remains explicitly unverified. No runtime changes, recording refresh, complete live qualifier, full gate, release approval, tag or publication belong to this diagnosis.
