---
base: ae9a1861
head: 8a9b4002
---

Adverse-event search now validates the complete FAERS, VAERS, recall, device,
and combined route matrix before provider contact. Count requires explicit
FAERS selection and zero offset; VAERS limits bound reactions; combined
FAERS-only filters leave VAERS visibly not requested; and device seriousness
has exact Death-or-Injury, Death, and Injury meanings in requests and output.

The implementation was split below source-size limits, and the shipped VAERS
and drug-helper specifications were aligned with the accepted route-specific
errors. Independent design and code reviews accepted the final behavior. The
complete release gate passed after the five-ticket batch.
