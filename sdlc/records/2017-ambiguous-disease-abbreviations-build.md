# 2017 build record — ambiguous disease abbreviations refuse in get disease

Landed 2026-10-08, merge d8afa297f. Review verdicts live on
sdlc/tickets/2017-ambiguous-disease-abbreviations-refuse-in-get-disease.md.

Outcome: two or more exact human holders refuse naming every candidate
with labels filled; non-human records never count (the veterinary
myeloma record); full words are ambiguous only by name (myeloma
resolves to multiple myeloma); the NCI trial search surfaces its
keyword degrade in the response; one-disease cards never mix another
disease's genes. Gates: branch CI green at 84a647904; yellow fully
green at the same sha; delta review ACCEPT. The landing's process
deviation (main before landcheck completion) is recorded on the ticket.
