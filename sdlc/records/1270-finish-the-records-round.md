---
base: 535747d9
head: c7b3a622
---

Finished the records round from the 2026-09-28 review file, as
ticket 1270 (documentation only; gated per the round's discipline).

The licensing registries now verify CIViC (docs FAQ, plain HTML:
CC0 1.0 Universal) and WikiPathways (terms.html: the CC0 waiver)
against pages fetched 2026-09-28, with the quotes in the evidence
table; PharmGKB points at the ClinPGx policy page; the retired
addresses (wiki classic, pharmgkb api, cpic /license/) are gone
from both registries. The pass record and the 2026-09-27 review
file carry the corrected counts — 45 verified, 2 changed, 1
unverifiable (Enrichr; warning 2027-01-15, fail 2027-03-21) — and
the retry owner names the developer agent working Ian's ordered
BioMCP queue.

Both review files carry complete dispositions naming the fixing
commits and the yellow-gate SHAs; the 2026-09-26 disposition is
rewritten unwrapped with commit citations; ticket 1260 names its
code paths and the prior experiment evidence; the README landing
contract computes the source count from sources.json so it cannot
drift. The dry-run lane is decided against, citing Ian's 2026-09-28
direction (PyPI-last makes a failed tag run retriable).

Evidence: yellow gate at c7b3a622 — lint, test, spec, stress all
OK, zero failed lines.
