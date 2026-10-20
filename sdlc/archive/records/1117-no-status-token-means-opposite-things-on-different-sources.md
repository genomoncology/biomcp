---
base: 7fd1d591a04ad10b86052584376ce136ab6c0df8
head: 7e2250b5095ee22f64a46374bda5c639010d9da0
---

# Refuse ambiguous active trial status

BioMCP now refuses `--status active` for both trial providers and names the
unambiguous replacement tokens. This prevents a reasonable status term from
silently selecting opposite recruitment states across providers.
