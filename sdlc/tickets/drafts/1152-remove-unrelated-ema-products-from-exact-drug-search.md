---
flow: quickfix
priority: 7
---

# Remove unrelated EMA products from exact drug search

## Goal

An exact drug-name search does not return unrelated EMA products because a chemical synonym contains a common word. On 2026-09-04, `biomcp --json search drug eflornithine --region all --limit 2` returned 279 European matches. Vaniqa matched eflornithine. Prasugrel Viatris appeared second even though its name, active substance, and indication contained no eflornithine. The reproduction and code evidence came from `sdlc/issues/2026-09-04-chemical-synonym-tokens-flood-ema-drug-search.md` in commit `f8ff2a78`.

## Desired functionality

An exact drug-name query returns EMA products whose product name, active substance, or trusted drug identity matches the requested drug. Chemical and systematic names do not create matches through isolated common tokens. Structured results explain which drug identity matched. Vetted special aliases retain their existing coverage without admitting unrelated products.

## Success criteria

- The European eflornithine results include Vaniqa.
- The European eflornithine results exclude Prasugrel Viatris and other products matched only through a common chemical-name token.
- Exact product, active-substance, and trusted alias matches retain their existing order.
- Structured output identifies the matching name and match type.
- Existing vaccine alias behavior remains covered without accepting unrelated one-token matches.
- Explicit indication searches retain their current behavior.

## Boundaries

This ticket corrects EMA drug-name matching. It does not redesign indication search, change the EMA data feed, remove trusted aliases, or alter U.S. drug results.
