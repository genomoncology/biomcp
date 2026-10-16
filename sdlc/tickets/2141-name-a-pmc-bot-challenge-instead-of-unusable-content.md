---
flow: build
priority: 3
deps: []
---

# 2141: Name a PMC bot challenge instead of calling it unusable content

Status: OPEN.

Milestone: 0.9.3

## Outcome

When PMC serves a bot-challenge page in place of an article, the full-text result says a bot challenge blocked it, so users and agents know a retry may work.

## Evidence

Reported 2026-10-16 by a literature probe of 0.9.2.

- Two runs of `biomcp --json --no-cache get article 18178632 fulltext` gave full text from PMC HTML once and `abstract_only` with `unusable_content` once.
- On the failed run PMC served a page titled "Checking your browser - reCAPTCHA".

## Work

- Recognize the challenge page in the PMC HTML step and report it as its own reason. Keep the next source in the chain running.

## Acceptance

1. A recorded-reply test with the challenge page reports the bot-challenge reason; it fails on current main.
2. The changelog carries a bullet for this ticket.
