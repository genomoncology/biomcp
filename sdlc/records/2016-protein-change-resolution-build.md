# 2016 build record — protein-change resolution keeps the named variant

Landed 2026-10-08, merge 04b7f6c79. Review verdicts live on
sdlc/tickets/2016-protein-change-resolution-keeps-the-named-variant.md.

Outcome: the named protein change outranks ClinVar-recorded lookalikes;
MANE numbering (derived from the ClinVar-preferred transcript) wins the
headline with a past-insert case pinned; a request whose reference
residue matches the MANE protein but whose change has no record refuses
with the protein, residue, lookalike, and retry form instead of wearing
a false other-transcript note (TP53 R209Q/G112D/R174H pinned; forty
positions live-verified); the honest notes survive with graceful
degradation. Gates: branch CI green at 86aadbc88; fix-round-3 review
ACCEPT; merged-tree CI green at 04b7f6c79 after reconciling the 1291
extraction's inventory change and the 2029 fixture constants.

## Fix round 4 (2026-10-09, landed e99ed6cfb)

The naming tier (the SnpEff annotation spelling the request ranks
between the MANE marker and first-NM_) resolves ClinVar-free BRCA1
requests honestly; the residue check runs before any note and notes
stay silent when unverifiable; transcript versions unify to
ClinVar's; the refusal retry names the requested change; the synthetic
fixture identifiers are invented; the P38398 capture makes the BRCA1
residue checks run in the spec lane.
