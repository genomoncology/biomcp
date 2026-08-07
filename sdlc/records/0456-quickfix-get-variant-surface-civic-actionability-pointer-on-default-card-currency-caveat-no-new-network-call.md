---
base: a2754cfdb34f3e0479f9d8e03cdbe4dc5fe448ee
head: cbb12088fccacb157c8fe98117c40033515ff4da
---
For an oncology variant the decision-relevant data is the **predictive / therapeutic** evidence, but the default `get variant` card never surfaces it. Running `biomcp get variant "EGFR L858R"` (no section) renders ClinVar + computational predictors (CADD/SIFT/PolyPhen, `src/render/markdown/variant.rs:151–162`, default-card only) and **omits CIViC entirely** — CIViC is shown only when explicitly requested or via `all` (template `templates/variant.md.j2:115–150`). So an agent that takes the default card gets pathogenicity predictors and no actionability, and only discovers the 49 CIViC evidence items / 3 assertions (therapies + AMP levels) if it already knows to run the `civic` section. The most clinically important signal is a tier below the least.

Imported from March ticket 456. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/456-quickfix-get-variant-surface-civic-actionability-pointer-on-default-card-currency-caveat-no-new-network-call
