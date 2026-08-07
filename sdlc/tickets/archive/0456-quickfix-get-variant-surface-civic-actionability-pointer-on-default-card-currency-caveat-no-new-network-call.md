---
flow: quickfix
priority: 5
---
# Quickfix: get variant — surface CIViC actionability pointer on default card + currency caveat (no new network call)

For an oncology variant the decision-relevant data is the **predictive / therapeutic** evidence, but the default `get variant` card never surfaces it. Running `biomcp get variant "EGFR L858R"` (no section) renders ClinVar + computational predictors (CADD/SIFT/PolyPhen, `src/render/markdown/variant.rs:151–162`, default-card only) and **omits CIViC entirely** — CIViC is shown only when explicitly requested or via `all` (template `templates/variant.md.j2:115–150`). So an agent that takes the default card gets pathogenicity predictors and no actionability, and only discovers the 49 CIViC evidence items / 3 assertions (therapies + AMP levels) if it already knows to run the `civic` section. The most clinically important signal is a tier below the least.

Completed under March on 2026-06-29, as March ticket 456. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/456-quickfix-get-variant-surface-civic-actionability-pointer-on-default-card-currency-caveat-no-new-network-call
