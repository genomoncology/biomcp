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
