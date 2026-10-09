# 2043 — Drug labels match the brand or the plain ingredient

Status: OPEN.

Milestone: 0.9.2

## Outcome

`biomcp get drug <brand>` shows that brand's own label. `biomcp get drug <ingredient>` shows the plain single-ingredient product, never a combination, biosimilar or under-the-skin form, and names the card for that ingredient. This finishes the label half of 2037's Outcome: "with that product's label" and "a card never takes another product's name or label".

## Evidence

Filed 2026-10-09 from the independent review of ticket 2037 as landed on main at `ec8a72d0d`. 2037 fixed Tarceva, Lartruvo, Portrazza, Lumoxiti and Zejula, and no brand in a 74-name sweep resolves on v0.9.1 but refuses on main.

- **Wrong labels.** Live on main, each brand and its ingredient name show another product's label. All match v0.9.1.

  | Query | Label shown |
  |---|---|
  | Keytruda, pembrolizumab | KEYTRUDA QLEX (with berahyaluronidase, under the skin) |
  | Herceptin, trastuzumab | OGIVRI biosimilar |
  | Rybrevant, amivantamab | Rybrevant Faspro (under the skin) |
  | Avastin, bevacizumab | Vegzelma biosimilar |
  | Rituxan, rituximab | Ruxience biosimilar |

  `src/entities/drug/get.rs:799` looks up the label by `drug.name`, never by the brand the user asked for, and `src/entities/drug/label.rs:773` takes openFDA's first result. Gleevec, Zytiga, Revlimid, Pomalyst, Velcade, Xeloda, Iressa, Gilotrif, Nexavar, Sprycel and Tasigna get a generic label of the right drug, not the brand's.
- **niraparib.** `get drug niraparib` names the card "niraparib tosylate monohydrate and abiraterone acetate" and shows the AKEEGA label (8245a990), while `get drug Zejula` shows niraparib with the Zejula label. `name_extends_request` (`src/transform/drug.rs:432-439`) counts a combination as a qualified form, and `merge_mychem_hits` at `drug.rs:629` prefers it. `CHANGELOG.md:28` claims this never happens.
- **The deferral misread the Outcome.** 2037's build status defers the label fix because it would move Herceptin from OGIVRI to HERCEPTIN and so break "a brand that resolved in v0.9.1 still resolves". A Herceptin card with the Herceptin label still resolves; that move is what the Outcome asks for.
- **2037 landed without a recorded review.** The 2037 file still reads `Status: OPEN`, and `pm item` says "branch and build". "Review: accept" appears only in the merge commit message, and the ticket records no review of the branch head, which 2038 finding 8 requires.
- **The tests miss parts of the rescue guard.** These mutations pass every test: dropping the exact-match and canonical-identifier requirement on the rescue's top result (`get.rs:354-356`); adopting the rescue's candidate without checking that its own MyChem record names it (`get.rs:735`); removing the identity-record filter while the Rybrevant test, which the ticket says pins it, still passes.
- **The Zejula label test cannot catch a wrong label.** Its fixture (`src/entities/drug/test_support/brand_resolution.rs:320-371`) answers only "niraparib" and returns only the Zejula label, so openFDA ordering never comes into play.
- **Thin cards.** `get drug Darzalex` refuses and lists daratumumab regimens. Phesgo gets a card with no DrugBank identifier and no label. Both match v0.9.1.
- **Slower refusals.** The Ara-C and 5-FU refusals take 7 to 17 seconds on main, against 4 to 9 seconds on `e71ac046b`. 2037's extra rescue call adds about 3 seconds on some misses.

- Starts from: ticket 2037 and its landing `ec8a72d0d`.
- Keeps: 2031's and 2037's identity fixes, including Tarceva, Lartruvo, Portrazza, Lumoxiti, Zejula, Lonsurf, Tagrisso, Rybrevant, terfenadine, mannitol and 5-FU.
- Changes: pass the brand query to the label lookup and prefer a label whose `openfda.brand_name` equals it; for an ingredient query, prefer the label whose generic name is exactly the ingredient, and skip combinations and biosimilar-suffixed names; refuse a name extension containing " and " or a comma so salts and hydrates still count and combinations do not; skip the rescue lookup when the discover step already ran for the query; resolve Darzalex to daratumumab.
- Proof: recorded-reply tests pinning Keytruda, Herceptin, Rybrevant and Avastin to their own label set identifiers and niraparib to a niraparib card; a Zejula fixture that serves the Akeega label first; tests that fail under each of the three mutations above; a recorded review of the branch head in the ticket; a recorded live sweep of the 74 names.
- Defers: shared product names between unrelated drugs (2039), and the Akeega and Zejula cards sharing DB11793 and brand lists, which 2039 covers.
