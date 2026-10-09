# 2031 — Drug name lookup never swaps the drug

Status: complete.

Milestone: 0.9.2

## Outcome

`biomcp get drug <name>` returns the named drug or says it found no match. It never returns a different drug's card or label without saying so.

## Evidence

Filed 2026-10-08 from the third review of the work since v0.9.1 (ticket 2033, finding 2). Ticket 1300 did not cause this; its diff does not touch name resolution.

- Live on main `7d17fd099`: `get drug terfenadine` returns the fexofenadine hydrochloride card and fexofenadine's label.
- `get drug mannitol` returns a card named "analgesic" with a minor-burns label. Edetate disodium resolves to a urea foot cream, and ferric oxide to calamine.
- The 1300 review called terfenadine "honestly empty". That does not hold live.

- Starts from: the MyChem name resolution `get drug` uses, and ticket 1300's identity guard for labels.
- Keeps: correct resolution for drugs whose name or synonym matches, such as osimertinib, TAGRISSO, lazertinib, amivantamab and mobocertinib.
- Changes: accept a MyChem hit only when the query matches its name, a brand name or a listed synonym; otherwise report no match, or list the candidates with a note.
- Proof: terfenadine and mannitol return no wrong card, and a test pins each against recorded captures; the five correct resolutions above still pass.
- Defers: nothing.

## Head review (fourth-review fix round)

- Code re-review (full head through c0dc28edf, covering the discover
  rescue and the brand fix the prior ACCEPT missed): ACCEPT 2026-10-08.
  Verified: the TAGRISSO resolution through MyChem's brand fields
  against the real capture (get and search agree); the rescue-miss
  keeps the sparse card; the four pinned cases hold; the keep-list
  resolves live; the parent-acid nuance and the pooling residuals
  recorded. Two report-only P2s: no automated pin on the rescue-miss
  path (deferred to the merge-pooling lane the ticket names), and the
  capture-header minimization sentence under-describes the dropped
  non-name field blocks.

## Root cause

Diagnosed 2026-10-08 against live MyChem (`mychem.info/v1/query`) and openFDA.

`get drug` resolves the card through `direct_drug_lookup` → `transform::drug::select_hits_for_name` → `transform::drug::merge_mychem_hits`. Ticket 1300 guarded only the label fallback (`lookup_label_response`); the card path feeds the label lookup an already-wrong name. Three holes, all in the card path:

1. `direct_drug_lookup` sends MyChem an unscoped full-text query, so the hit list contains any drug whose record mentions the term (excipients, metabolite synonyms, combination products). The query term appearing in a record is not evidence the record is that drug.
2. `select_hits_for_name` admitted hits on a word-boundary name match and then fell back to *all* hits when nothing matched. A foreign synonym that merely extends the query ("Terfenadine carboxylate" for terfenadine — fexofenadine's DrugBank synonym) selected the fexofenadine record, and the richness sort put it ahead of terfenadine's own record. When nothing matched at all, the all-hits fallback merged whichever text hit was richest.
3. `merge_mychem_hits` named the card from the hit's *first* NDC nonproprietary name, regardless of the query. MyChem merges NDC product rows into the ingredient's record, and the first row is often another product: the mannitol record's first NDC name is "Analgesic" (mannitol is an inactive ingredient there), edetate disodium anhydrous's is "urea, glycerin, aloe, disodium edta", ferric oxide's is "Calamine and Zinc Oxide". The card then carried that name and the label lookup fetched that product's label (minor-burns analgesic, urea foot cream, calamine).

Live evidence (recorded 2026-10-08):

- `q=terfenadine`: hit 1 is terfenadine (DrugBank DB00342, "Terfenadine"); hit 2 is fexofenadine (DB00950, synonyms "Terfenadine carboxylate", "Carboxyterfenadine", "Terfenadine acid metabolite"). Hole 2 selected both; the richer fexofenadine hit named the card "fexofenadine hydrochloride".
- `q=mannitol`: hit 1 is mannitol itself (DB00742), but its NDC rows begin "Analgesic", "Analgesic", … before "mannitol". Hole 3 named the card "analgesic".
- `q=edetate disodium`: the only name-bearing hit is "Edetate disodium anhydrous" (DB14600) whose NDC rows are "urea, glycerin, aloe, disodium edta". Hole 3 named the card after the foot cream.
- `q=ferric oxide`: the ferric oxide record (DB11576) carries NDC rows "Calamine and Zinc Oxide", "Calamine Plus Pramoxine HCL", …, "FERRIC OXIDE RED". Hole 3 named the card "calamine and zinc oxide".

## Success criteria

1. A hit is admitted only when the query matches the hit's own name, a brand name, or a listed synonym. Exact matches are preferred: when any hit's name equals the query, only exact hits merge. Qualified forms of the same drug that lead with the query (salt, hydrate: "edetate disodium anhydrous", "Imatinib Mesylate") still match when no exact name exists; a name that merely contains the query somewhere inside it does not. The all-hits fallback is gone.
2. The card name comes from a name field that itself matches the query, not the hit's first NDC row; a non-matching brand never names the card while a generic identity exists (Keytruda still canonicalizes to pembrolizumab).
3. When no hit matches by name, resolve the query through openFDA's field-scoped label search before refusing — a secondary path only, since many live label records carry no openFDA block. A miss that survives refuses honestly, naming what the MyChem text search matched. Brand names resolve through MyChem itself: `MYCHEM_FIELDS_GET` fetches `drugcentral.synonyms` and `ndc.proprietaryname`, so a brand query selects the drug record the brand lives on — TAGRISSO lands on osimertinib (DB09330) that way, agreeing with `search drug Tagrisso`.
4. `get drug terfenadine` returns terfenadine's own card (DB00342) with an honest empty label ("No openFDA SPL label record matched this drug."); fexofenadine appears nowhere on it. `get drug mannitol` returns mannitol's own card named "mannitol" (DB00742). `get drug edetate disodium` returns the exact ChEMBL/UNII edetate disodium record (CHEMBL3989507) — not the anhydrous qualified form and never the urea foot cream. `get drug ferric oxide` returns ferric oxide's own record, never calamine. Each is pinned by a test against the recorded MyChem capture.
5. osimertinib, TAGRISSO, lazertinib, amivantamab and mobocertinib still resolve correctly (amivantamab through its exact DrugBank record DB16695; TAGRISSO through the MyChem brand fields — see criterion 3; lazertinib verified live — see the Build status).

## Build status

- Built on branch `tickets/2031-drug-name-lookup-never-swaps`, fix `d731a1ad7` plus pin update `695ecc9b1` (tip), 2026-10-08.
- `src/transform/drug.rs`: `select_hits_for_name` keeps exact-name hits, falls back to hits whose name leads with the query (salt, hydrate, combination) only when no exact name exists, and no longer falls back to every text hit. `merge_mychem_hits` names the card from the first canonical field (every NDC row, every openFDA generic, DrugBank, ChEMBL, GtoPdb, UNII, ChEBI) that itself matches the request; a brand that matches while no canonical name does keeps the generic identity (Keytruda still canonicalizes to pembrolizumab).
- `src/entities/drug/get.rs`: when no hit matches by name, `get` resolves the query through openFDA's own identity fields (`openfda.generic_name`/`brand_name`, one candidate, then a named re-lookup) before refusing; a surviving miss returns a NotFound that names up to three drugs the MyChem text search matched.
- Pinned against MyChem captures recorded 2026-10-08 (`src/transform/drug/name_resolution_tests.rs` and `src/entities/drug/test_support.rs`): terfenadine selects only DB00342 with an honest empty label; mannitol names the card "mannitol" with no "analgesic" field; edetate disodium takes the exact ChEMBL/UNII record (never the urea cream); ferric oxide takes DB11576 (never calamine); a text-only match refuses naming what matched; TAGRISSO lands on osimertinib through the openFDA fallback; amivantamab keeps DB16695; the imatinib card keeps its "imatinib mesylate" established name.
- Verified on the build host in a dedicated worktree at `695ecc9b1`: cargo nextest drug scopes 145/145; `make spec` all routine pages green (drug.md 17, drug-label-sections.md 7, drug-interactions.md 6, section-outcomes.md 15, trial-intervention-aliases.md 5, every other page green); `tools/check-quality-ratchet.sh` and `tools/check-test-wait-ratchet.py` pass; the packaged file count is 1406 exactly.
- CI on `695ecc9b1` (run 37797398268): `make lint` green (fmt, clippy, deny, both ratchets), every job green except canonical-gates, which hit the 45-minute cap inside `make test` — the ticket-2030 article test hang that stalls main's CI, not this change. The first push's lint failure was the exact source-size baselines; `695ecc9b1` repins `src/transform/drug.rs` at 1226 lines and `src/entities/drug/get.rs` at 1201 (ticket 2031 authorized increases) and raises the packaged file cap to 1406.
- Residual, accepted: when no record carries the exact name, a record whose name merely leads with the query can still resolve, named by its own longer name (the salt/hydrate allowance); MyChem text matches never merge silently. The 0.9.2 changelog bullet belongs to 1305's changelog lane.
- Keep-list live check, 2026-10-08, branch binary `0.9.1+g97eab4ad` (build-host build of this branch, live MyChem/openFDA): `get drug lazertinib` returns lazertinib's own card — named "lazertinib", DrugBank DB16216, small-molecule, EGFR mechanism and targets EGFR/JAK3/KDR/SYK, CIViC variant targets EGFR L858R OR EGFR Exon 19 Deletion. No other drug's field appears. The other four keep-list drugs are pinned by tests (osimertinib and mobocertinib by `spec/entity/drug-label-sections.md`, TAGRISSO and amivantamab by capture tests).

## Fix round (review, 2026-10-08)

- Code review: ACCEPT 2026-10-08 on `695ecc9b1`, three P2s folded in this round (c72a8ac91): Success criterion 4's edetate wording now names the exact ChEMBL/UNII record the build selects (CHEMBL3989507); the lazertinib keep-list claim carries the dated live check above; and `discover_sparse_drug_rescue`'s Canonical branch adopts a candidate only through `named_drug_response` — its own MyChem names must match — so a rescue miss keeps the query-matched sparse card instead of building a name-only card from another record's text hits. The get.rs source-size baseline moves with it to 1203 lines (delta 100).
- Coverage note: the ACCEPT above reviewed `695ecc9b1`; the later commits (the discover-rescue change and the fourth-review brand fix) await the fresh pre-landing review.

## Fix round (fourth review, ticket 2035, 2026-10-08)

- Finding 1 (blocker): `get drug Tagrisso` refused live because the openFDA fallback relied on `openfda.brand_name:"TAGRISSO"`, for which live openFDA answers NOT_FOUND (the label carries no openFDA block); the passing e2e test had invented that openFDA reply. Fixed: `MYCHEM_FIELDS_GET` now fetches `drugcentral.synonyms` and `ndc.proprietaryname` (`src/sources/mychem.rs`), `hit_all_names` admits them for selection (`src/transform/drug.rs`), and the TAGRISSO capture is re-recorded from the real MyChem reply — the DrugCentral synonym "tagrisso" and the NDC proprietary names name the osimertinib record itself, so the brand resolves deterministically through MyChem, with the e2e test answering every openFDA label search with no match. `search drug Tagrisso` and `get drug Tagrisso` agree on osimertinib.
- The `openfda_label_identity_candidate` comment no longer claims an exact identity match the code does not require (finding 20): it now says the candidate is the first row of the field-scoped search, which can miss when records carry no openFDA block.
- Finding 20, recorded, predates this lane or stays open: `get drug Rybrevant` intermittently lost DB16695 because the upgrade runs through the OLS4-backed discover rescue, which predates 2031 — with this round's brand fields the no-OLS4 floor is the deterministic sparse `amivantamab-vmjw` record (exact NDC proprietary name "Rybrevant") instead of an intermittent card; the OLS4 intermittency itself is the discover lane's, not this one. The "Pain Relief" shared-product-name merge (one card pooling acetaminophen's DB00316 with TRPV1 targets from other records sharing the NDC product name) and the trastuzumab card listing Enhertu as a brand both reproduce on main and predate 2031; both stay open for the merge-pooling owner. The tier-2 admission of "Terfenadine carboxylate" is the recorded salt/hydrate residual above.
- Rebase note (2035 finding 2): `tests/test_source_package_boundary.py` conflicts with main at `1_406`; the merged tree packages 1407 files, so the count becomes `1_407` at rebase. This branch's own tree stays at 1406.
- Verified at `0e65170e` on the build host (dedicated worktree, live sources): `get drug` returns terfenadine/DB00342, mannitol/DB00742, edetate disodium, ferric oxide red/DB11576, Tagrisso and TAGRISSO both → osimertinib/DB09330, Keytruda → pembrolizumab/DB09037, Gleevec → imatinib mesylate/DB00619, Enhertu → trastuzumab deruxtecan/DB14962, Lazcluze → lazertinib/DB16216, Rybrevant → amivantamab/DB16695; `search drug Tagrisso` lists osimertinib, so get and search agree. Drug scopes 145/145 and `make lint` green on the same tree.
- Live nuance, recorded: with `drugcentral.synonyms` fetched, the edetate-disodium card carries DrugCentral's salt synonym "edetate disodium" on the edetic-acid record (DB00974), so the card is named "edetate disodium" with the parent-acid DrugBank id beside the exact ChEMBL/UNII record — the same parent/salt relation as warfarin and warfarin sodium, not a wrong-drug card. The ferric-oxide card's Brand Names row can list "Calamine" because DrugBank's own synonym list for ferric oxide includes it; that is pre-existing synonym rendering, the same class as Enhertu on the trastuzumab card.
