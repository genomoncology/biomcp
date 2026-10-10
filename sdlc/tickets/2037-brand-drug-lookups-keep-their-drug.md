# 2037 — Brand drug lookups keep their drug

Status: OPEN.

Milestone: 0.9.2

## Outcome

`biomcp get drug <brand>` returns the drug that brand names, with that product's label, or an honest refusal. A brand that resolved in v0.9.1 still resolves. A card never takes another product's name or label.

## Evidence

Filed 2026-10-09 from the pre-tag review of main at `e71ac046b` (ticket 2038, finding 3).

- `get drug Tarceva` refused 3 of 3 times on main. `cf8113f3a` and v0.9.1 return erlotinib hydrochloride (DB00530) with the erlotinib label. Lartruvo (olaratumab), Portrazza (necitumumab) and Lumoxiti (moxetumomab pasudotox) regress the same way. Live MyChem holds these brands only on a record with no name. `src/entities/drug/get.rs:712-727` tries openFDA, finds no label, and returns `name_miss_not_found` before the discover rescue runs. The refusal lists "erlotinib hydrochloride (CHEBI:53509)" as its top possible match.
- `get drug Zejula` returns a card named "niraparib tosylate monohydrate and abiraterone acetate" with the AKEEGA label (set ID 8245a990…). v0.9.1 and `cf8113f3a` do the same. `merge_mychem_hits` (`src/transform/drug.rs:572-578`) falls back to the first openFDA generic name or NDC row. MyChem pairs `proprietaryname: ZEJULA` with `nonproprietaryname: niraparib` on the same rows. This contradicts the changelog line "a card never takes its name from an unrelated product's packaging row" (`CHANGELOG.md:28`).
- Keytruda and pembrolizumab return the KEYTRUDA QLEX label (pembrolizumab with berahyaluronidase alfa). Herceptin and trastuzumab return the OGIVRI biosimilar label. Rybrevant and amivantamab return the Rybrevant Faspro label. All match v0.9.1.
- `get drug Lonsurf` is now named "lonsurf"; `cf8113f3a` said "trifluridine and tipiracil".
- Removing DrugCentral synonyms from `hit_all_names` (`src/transform/drug.rs:170`) passes every test, though 2031 credits them for the Tagrisso fix. Unguarding the discover rescue (`get.rs:755`) passes every test.

- Starts from: ticket 2031 and its landing `548f0b854`.
- Keeps: 2031's fixes for terfenadine, mannitol, 5-FU, Tagrisso and Rybrevant; the name, brand and synonym rule.
- Changes: run the guarded discover rescue before a name-miss refusal; name a brand card from the nonproprietary name MyChem pairs with that brand; choose the label whose brand matches the query, and prefer the plain product over combinations, biosimilars and subcutaneous forms when the query is the ingredient name.
- Proof: recorded-reply tests for Tarceva and Zejula that fail on `e71ac046b`; tests that bite when DrugCentral synonyms or the rescue guard are removed; a recorded live sweep of the oncology brands that resolved in v0.9.1.
- Defers: product names shared by unrelated drugs, such as "Pain Relief", unless a ticket is filed for them.

## Build status

- Built on branch `tickets/2037-pre-tag`, tip `9e2784db8`, 2026-10-09. Two commits carry the change: `a1ecb5cf9` plus `f45b5b2a` (the brand-pairing refinement), `9e2784db8` splits the new tests into their own module and repins the source baselines.
- `src/entities/drug/get.rs`: the guarded discover rescue now runs before a name-miss refusal. When no MyChem record names the query and openFDA's identity fields miss, the rescue resolves the query and adopts its candidate only through `named_drug_response`; every surviving miss keeps the honest refusal that names what the text search matched. A refusal now costs one extra OLS4 lookup (5-FU and Ara-C still refuse).
- `src/transform/drug.rs`: a brand card is named from the nonproprietary name MyChem pairs with that brand on its own NDC row (`brand_paired_nonproprietary_name`). A canonical name that extends the query ("imatinib mesylate") keeps naming the card; a canonical name that merely equals the brand string (a bare CHEBI record named "Lonsurf") yields to the pairing; the pairing counts only on identity-bearing records, so a naked product row (Rybrevant's "amivantamab-vmjw") never renames the identity record's card.
- Tests: recorded-reply pins recorded 2026-10-09 from live MyChem, openFDA and OLS4 (trimmed as each capture's header documents) — Tarceva resolves erlotinib hydrochloride (DB00530) with the erlotinib label through the rescue; Zejula names niraparib with the Zejula label (set b7f675e2), never the Akeega row or label (8245a990); Lonsurf says trifluridine and tipiracil; Keytruda keeps pembrolizumab; Rybrevant keeps amivantamab; a capture holding Tagrisso only as a DrugCentral synonym bites when the synonyms leave `hit_all_names`; a two-exact-drugs brand bites when the rescue guard's competing-exact check is removed.
- Red on main: branch `tickets/2037-red-proof` held main `203daec02` plus the test files alone. The Tarceva, Zejula (card and transform pin) and Lonsurf tests failed there — Tarceva refused, Zejula named "niraparib tosylate monohydrate and abiraterone acetate" with the Akeega label, Lonsurf named "lonsurf" — while the Keytruda, Rybrevant and Tagrisso pins passed. On the fix branch all nine pins pass, the drug and discover scopes pass 391/391, the full nextest lane passes 4097 with 33 skipped, and `make lint` is green after repinning `src/transform/drug.rs` at 1309 lines (delta 309) and `src/entities/drug/get.rs` at 1231 (delta 128).
- Live sweep, 2026-10-09, branch binary on the build host, live sources, cache off: Tarceva→erlotinib hydrochloride DB00530 (label ab6f3cb3), Lartruvo→olaratumab DB06043, Portrazza→necitumumab DB09559, Lumoxiti→moxetumomab pasudotox DB12688, Zejula→niraparib DB11793 (b7f675e2), Lonsurf→trifluridine and tipiracil DB00432, Keytruda→pembrolizumab DB09037 (097d166f, QLEX, as v0.9.1), Herceptin→trastuzumab DB00072 (b6465b44, OGIVRI, as v0.9.1), pembrolizumab and trastuzumab unchanged, amivantamab and Rybrevant→amivantamab DB16695 (9e58b045, Rybrevant Faspro, as v0.9.1), Tagrisso→osimertinib DB09330, Gleevec→imatinib mesylate DB00619, Enhertu→trastuzumab deruxtecan DB14962, Lazcluze→lazertinib DB16216, Opdivo→nivolumab DB09035, Lynparza→olaparib DB09074, Ibrance→palbociclib DB09073, Revlimid→lenalidomide DB00480, Xeloda→capecitabine DB01101, Kadcyla→ado-trastuzumab emtansine DB05773, Imbruvica→ibrutinib DB09053, Tecentriq→atezolizumab DB11595, Kisqali→ribociclib DB11730, Verzenio→abemaciclib DB12001, Tukysa→tucatinib DB11652, carboplatin DB00958, cisplatin DB00515, paclitaxel DB01229, docetaxel anhydrous DB01248, terfenadine DB00342 (no label, honest), mannitol DB00742, cisapride DB00604; 5-FU and Ara-C refuse honestly.
- The Changes bullet on label choice is not met, recorded reason: on live openFDA (checked 2026-10-09) any query-aware brand match or plain-product preference that makes Zejula's label robust also moves Herceptin OGIVRI→HERCEPTIN, Keytruda QLEX→KEYTRUDA, and Rybrevant Faspro→plain RYBREVANT, and each move contradicts the Outcome's rule that a brand which resolved in v0.9.1 still resolves and the ticket's own Keeps pins. Both named regressions are fixed without it, and Zejula takes the Zejula label through the card-name fix alone. Residual, accepted: the label choice still follows openFDA's effective_time order once the card name is right, so a newer competing label could outrank the brand's own; that residual belongs to the label-choice bullet deferred here.

## Review status (2026-10-10)

The pre-landing review record added here earlier (a verdict on
`ea7f9016a` from a reviewer who ran no commands, endorsing the label
deferral) does not satisfy the landed-head requirement: it predates the
landing and the clean-release ruling now overrides the deferral it
endorsed. A fresh review of the landed head `ec8a72d0d` follows with
2043's label work, recorded through pm.
