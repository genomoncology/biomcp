---
priority: 1
---

# 2281 — Use product headers in replay live checks

Status: OPEN.

Milestone: 0.9.4

## Outcome

The release replay checker sends the honest application-controlled headers of the current public BioMCP client. A controlled HTTP consumer proves the request profile. The seven previously denied Reactome and WikiPathways recordings receive one bounded recheck with that profile and truthful per-entry observations.

Changes: `scripts/replay_drift.py`, `tools/check-replay-live`, `tests/test_replay_live_drift.py`, `testdata/replay/manifest.json`, `CHANGELOG.md`, `sdlc/records/2281-checker-request-fidelity.md`.

## Evidence and authorization

Release collection 2258/1792447971557-46385 received seven HTTP 403 bodies containing error code 1010. Experiment 2280/1792448958050-39962 verified the official released 0.9.3 binary and successfully ran one Reactome and one WikiPathways command with normal client identity. WikiPathways used a lookup endpoint, so search health remains unverified. The collection used Python urllib defaults; the product uses its declared versioned identity and client headers. User-Agent alone is not proven causal. The PM explicitly approved matching the checker to public product headers and rechecking those seven recorded requests once. Private primary evidence remains in experiments 2096 and 2099 on the build machine.

## Implementation and proof

Before code, restate the outcome and reproduce the current header mismatch against a controlled local HTTP server through pm job. Read the actual shared client header construction and derive the current version from canonical metadata. Match application-controlled headers, including User-Agent, Accept and no-cache behavior where the public client defines them. Describe remaining transport-default differences honestly; do not claim identical network fingerprints. Preserve strict request identity, authentication refusal, source restrictions, no redirect/proxy/retry, caps and all existing checker controls. No new transport dependency or broad refactor.

Run focused controlled red/green tests through pm job. After that, one paced request per each of the seven previously denied recorded requests is authorized, with retained raw bytes, method, URL, headers, status, hashes and checked source revision. Do not call any other provider. Do not inspect or add credentials; only NCI_API_KEY may be inherited if existing job machinery requires it, never printed or stored. No Semantic Scholar request remains authorized. KEGG remains off.

If the seven now return valid data, compare faithfully and report any real drift; do not invent payload fixes or broaden this ticket. If still denied, retain explicit not-checked-live provider-denied evidence for the coordinator. Never count denial as checked or passing. Public record contains safe technical facts, raw replies remain private. Push once after controlled checks pass, get fresh review, stop without landing. This checker implementation change must land alone because it changes gate code.

## Delivery proof

Developer stopped at 7242453c8c86d7cdf3d93cc8ac842a02bffd9671. Controlled HTTP red 2281/1792449416798-65323 proves the old identity mismatch; focused green 1792449434355-71721 passes 31 cases, followed by final formatting/quality/security checks in 1792449515653-91782 and 1792449657161-55428. Seven authorized requests in 1792449561439-4540 all return 200, but only two original fixture comparisons pass; three drift and two lack historical metadata. The safe record retains exact keys/hashes, transport limits and source terms. No manifest/body refresh occurred. Separate 2282 owns those observations and consumer-safe fixture repair. No further provider calls are authorized.

Keep: experiments/2100-biomcp-2281-checker-request-fidelity — retain authorized request headers, raw replies, hashes and comparison evidence for release fixture repair and review.
