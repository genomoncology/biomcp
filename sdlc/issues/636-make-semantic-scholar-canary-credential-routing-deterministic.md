# Make Semantic Scholar canary credential routing deterministic

Severity: nice-to-have.

Carried over from March, where it was raised against ticket 636
on 2026-07-31 and left open. The text
below is as filed.
The live article-graph page cannot reliably prove its stated `S2_API_KEY` dependency: during verification, its unauthenticated Semantic Scholar detail and recommendations requests both returned HTTP 200 and the entire page passed 11/11 with `S2_API_KEY` unset.

The live provider receipts were `GET /graph/v1/paper/PMID:23450558?...` → 200 and `GET /recommendations/v1/papers/forpaper/97ae9501d5f7f5ddc2d38ea98abdca2dc4939d42?...` → 200 for both authenticated and unauthenticated requests. The ticket's historical 429 is therefore a transient provider state, not a reproducible contract.

Suggested action: add a deterministic `test` or `spec` fixture that captures the `x-api-key` header and verifies the live-canary command uses the raw release binary (key present), while `tools/biomcp-ci` deliberately omits it. Keep the live page as a provider smoke test rather than treating an intermittent anonymous 429 as a required red proof.
