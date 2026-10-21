# 2328 — Return BLENREP approvals for belantamab mafodotin

Status: OPEN.

Milestone: 0.9.5

## Outcome

`biomcp get drug "belantamab mafodotin" approvals` returns BLENREP (BLA761440) when a source-confirmed BLENREP identity's ingredient metadata agrees, as sdlc/decisions/2026-10-19-biologic-approvals-match-the-exact-product.md requires. A suffix or shared core alone still never matches. When the identity cannot be confirmed, the output says so and names the provider record it found, instead of returning approvals:[] silently.

- Proof: outside-in replay tests show BLA761440 for belantamab mafodotin, BLA761036 still for daratumumab, BLA761145 still excluded for plain daratumumab, and an unconfirmed suffixed product reported as unconfirmed rather than matched.

## Evidence

- On 2026-10-21, the research team reran its logged errors on the 0.9.4 rehearsal binary cdb9f4a1b. Item R10, from 2270: daratumumab now returns BLA761036, but belantamab mafodotin still returns approvals:[] with a suffixed-name identity restriction, and the provider's record is not returned.
- The ruling names BLENREP (BLA761440) as matching belantamab mafodotin through a source-confirmed identity. The 0.9.4 code does not find that identity.
- Clinical questions about the match go to a clinical reviewer.
