# citation-evidence fails on every pair and names the wrong source

Filed 2026-10-02 from experiment 421 (`experiments/421-rank-then-read-biomcp-literature`), run against the published 0.9.1 wheel.

## What happens

`biomcp article citation-evidence <citing> <cited>` fails immediately on every pair tried, including pairs of real PMIDs (35063965→31392741, 35063965→29735000, 33199365→25910677), from a clean network. The failure is identical each time:

```json
{
  "_meta": { "not_found": false },
  "error": {
    "code": "api",
    "message": "API request to BioMCP source failed.",
    "recovery": "Review source configuration and retry.",
    "source": "BioMCP source"
  },
  "passages": []
}
```

## Two defects

1. The path is broken or its upstream is consistently down — three pairs, three instant identical failures, while every other article command (search, get) works from the same install.
2. The error's `source` field says "BioMCP source" instead of the actual provider (Semantic Scholar or OpenCitations). Ticket 1242's work made failing sources render honestly with the provider named; this path masks it, so a user cannot tell which service to check. The recovery hint ("Review source configuration") is also wrong — no user configuration exists for this path.

## Expected

A real citation edge returns its bounded passage (or the `reference_confirmed_without_passage` outcome); a nonexistent edge returns an honest no-such-edge answer; a provider outage names the provider.

## Reproduce

`biomcp article citation-evidence 35063965 31392741 -j` on the 0.9.1 wheel.

## Diagnosis, 2026-10-02 (experiment 421 follow-up)

`RUST_LOG=debug` shows the command's only network hop is `api.semanticscholar.org`, and direct probes of the Semantic Scholar graph API from this network answer HTTP 429 (rate limit) while every Europe PMC path works. So the proximate cause is Semantic Scholar refusing unauthenticated traffic, and the two defects stand: the command has no degradation path when its first hop is refused (no OpenCitations-first fallback, no honest "provider rate-limited" answer), and the rendered error names "BioMCP source" instead of the provider. A user on a rate-limited network sees a configuration error for a condition they cannot configure around.
