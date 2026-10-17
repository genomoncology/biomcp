---
flow: build
priority: 3
deps: []
---

# 2141: Name a PMC bot challenge instead of calling it unusable content

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision c706103, accept

## Outcome

When PMC serves a bot-challenge page in place of an article, the full-text result says a bot challenge blocked it, so users and agents know a retry may work.

## Evidence

Reported 2026-10-16 by a literature probe of 0.9.2.

- Two runs of `biomcp --json --no-cache get article 18178632 fulltext` gave full text from PMC HTML once and `abstract_only` with `unusable_content` once.
- On the failed run PMC served a page titled "Checking your browser - reCAPTCHA".

### Reproduction on current main (2026-10-17)

- Live, ticket command, installed 0.9.2 (`biomcp --json --no-cache get article 18178632 fulltext`, two runs from this session): both returned `full_text` from PMC HTML (`Europe PMC XML ... source_unavailable`, `NCBI EFetch PMC XML ... abstract_without_body`, `PMC OA Archive XML ... no_content`, `Europe PMC MED XML ... source_unavailable`, `PMC HTML ... body_detected`). The gate is intermittent; it served the article both times tonight.
- Deterministic, current main code (`6b4545ac3` = main + test-only): serving the recorded-shape challenge page (`testdata/sources/pmc_article/pmc-bot-challenge.html`, authored from the title evidence above) at the PMC HTML step makes the recorded attempt read `"reason": "unusable_content"` instead of a bot-challenge reason. macOS gate host job `1792207045377-82291`, command `cargo nextest run --locked --no-default-features --lib pmc_bot_challenge`: 1 failed — `assertion left == right failed, left: String("unusable_content"), right: "bot_challenge"`.

## Work

- Recognize the challenge page in the PMC HTML step and report it as its own reason. Keep the next source in the chain running.

## Acceptance

1. A recorded-reply test with the challenge page reports the bot-challenge reason; it fails on current main.
2. The changelog carries a bullet for this ticket.
