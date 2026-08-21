---
base: 30a8b6b5515b5590a8e72d06665a83cad8bdde3b
head: 7c2de3a49815d1c52da556d2673828f4beac2fa2
---

# Prove CAid-to-gene resolution for empty ERepo results

Empty single-CAid Markdown results now show a ClinGen Allele Registry gene when
the registry returns one. Missing or unavailable registry mappings preserve the
existing empty-result message, so callers retain the established fallback.
