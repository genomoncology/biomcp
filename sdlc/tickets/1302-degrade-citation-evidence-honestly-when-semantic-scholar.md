# 1302 — degrade citation-evidence honestly when Semantic Scholar refuses

Filed 2026-10-05 by the BioMCP 0.9 lead, promoting the issue of 2026-10-02 after reproduction; supersedes that issue.

Status: complete.
Milestone: 0.9.2

## Build status

- Built on branch `tickets/1302-citation-evidence-degradation`, sha `816f1d1e4` plus the review P2 fold, 2026-10-06.
- Code review: ACCEPT 2026-10-06.
- Code re-review (post-landing delta): ACCEPT 2026-10-07. The post-review commit 8e3c84498 (degraded no-edge fixture pin, the review's own P2) is pure strengthening: pins the NotFound shape, the provider naming, and the single Semantic Scholar seed request. No pins loosened, no messages softened. The one P2 (the degraded no-edge NotFound shape unpinned) folded as one loopback case before CI. Verified clean: only 429/5xx refusals degrade, transport and decode errors propagate; the fallback cannot fabricate an edge (fail-closed on shape-invalid rows); DOI resolution goes through Europe PMC exact-ID queries; the refused first hop makes exactly one Semantic Scholar request (429 converts before retry middleware); no message names "BioMCP source" or leaks upstream bodies; the error projection case is narrow; source-size repins accurate.
- Live proof against real providers: a real Semantic Scholar 429 answered from OpenCitations in 2.66 s wall, 0.03 s user CPU (the old binary measured 22.12 s against the same refusal); both-refused pairs answer in 1.24 s naming both providers and reasons.
- Deferred: degraded results skip the sidecar (no S2 paper IDs to key on; one cheap index request recomputes); the every-provider-refused retry ladder (~8 s wall, no CPU burn) belongs to the retry-policy work under 1299.

## Outcome

## Outcome

`article citation-evidence` answers from OpenCitations when Semantic Scholar refuses, names the provider it used and the provider that failed, and stops burning tens of seconds of CPU on a refused request.

## Evidence

- Starts from: Reproduced 2026-10-05 on main a877443f: `article citation-evidence 35063965 31392741 -j` fails in 22 s wall (17 s user CPU) with `api / API request to BioMCP source failed`, `source: "BioMCP source"`. The issue's 2026-10-02 diagnosis holds: a direct probe of `api.semanticscholar.org/graph/v1` answers HTTP 429 in 0.13 s today for unauthenticated traffic, every other article path works, and the command's first hop is Semantic Scholar. An OpenCitations client already exists (`src/sources/opencitations.rs`, used for confirmation) but the command never reaches it when the first hop is refused. The error names "BioMCP source" instead of the provider (the generic naming in `src/error.rs`), and the recovery hint points at configuration no user has.
- Keeps: The passage-bounded evidence output, the `reference_confirmed_without_passage` outcome, and Semantic Scholar as the first source when it answers.
- Changes: (1) Fall back to OpenCitations-first when Semantic Scholar is refused (429/5xx), or retry the pair reversed, so the command answers when either provider works. (2) The error and the source status name the actual provider and the actual reason (rate-limited, unavailable), never "BioMCP source". (3) A refused first hop fails fast — no multi-second CPU burn after refusal; isolate the spin the same way ticket 1299 requires.
- Proof: Recorded fixtures: a 429 from Semantic Scholar with a working OpenCitations answers the edge; both refused names both providers and the reason; wall time for a refused request stays under a few seconds.
- Defers: Authenticated Semantic Scholar keys.
