# PMC3040717 fixture does not preserve proof-of-work coverage

On 2026-08-23, the stored-source fixture returned `healthy_absent` for both
PMC3040717 supplementary files through `get article 20516115 assets`. Ticket
1045 says those entries should retain `pmc_proof_of_work`; the fixture has no
handler for either `/articles/instance/3040717/bin/` URL. A separate PMC123466
fixture does exercise the proof-of-work outcome. Add the missing stored fixture
route and contract coverage outside ticket 1045's HTML-only test scope.
