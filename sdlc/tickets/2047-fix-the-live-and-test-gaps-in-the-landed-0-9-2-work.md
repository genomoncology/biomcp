# 2047 — Fix the live and test gaps in the landed 0.9.2 work

Status: OPEN.

Milestone: 0.9.2

## Outcome

Every user-facing answer that 0.9.2 changed is right on live sources, every fix the 0.9.2 records claim has a test that fails when the fix is removed, and the open items 2042, 2043 and 2044 left behind are done.

## Evidence

Filed 2026-10-10 from the independent review of main at `c6d972d47`: a live sweep of a release build on the build host against v0.9.1 with the cache off, and a code read of the 2042, 2043, 2044 and 2045 landings. Every item here holds the tag under Ian's clean-release ruling.

The live sweep confirms the named fixes: BRCA1 Y1866D, H1883D, Q1878R, S1587F, S1551Y, TP53 S183Y, R209Q and R116Q, the MANE current versions, and the Keytruda, Herceptin, Rybrevant, Avastin, Rituxan, Zejula, Tagrisso and Gleevec labels all answer correctly.

Live defects on main:

- `article entities 37887282` now prints `biomcp get disease MESH:…` links that open the wrong card. "Lung Cancers" links to `MESH:D008175`, which opens `lung benign neoplasm (MONDO:0002732)`. `MESH:D000230` for "adenocarcinomas" opens granular cell carcinoma, `MESH:D000236` for "adenomas" opens papillary adenoma, and `MESH:D009369` for "cancer" opens disease of cellular proliferation. v0.9.1 printed search commands here. This is new in 0.9.2.
- Darzalex and daratumumab return the DARZALEX FASPRO label (set `4bb241af`), while DailyMed holds the plain DARZALEX label (set `a4d0efe9`). It is the plain-versus-Faspro mix-up main fixed for Rybrevant. The card's brands omit Darzalex itself.
- Every drug card now prints an empty `## FDA Label` heading, on the command line and over MCP. v0.9.1 printed none.
- `get drug 5-FU` refuses honestly but takes 7 to 17 seconds against 2 in v0.9.1, and lists `Fluorouracil (Gene, MESH:D005472)` as a gene.
- Phesgo has no label, no DrugBank identifier and no safety line, while DailyMed holds PHESGO (set `27dd5e6b`). 2043's Evidence named it, and neither its Changes nor its build record answers it.
- Live `get variant <rsid> all` fails on the retired GWAS Catalog endpoint (HTTP 410), as the 2042 build record notes. `src/entities/variant/gwas.rs:511-517` degrades only on "source unavailable". Commit `0ec8776e0` serves an empty page offline, so no test sees it.

Gaps in 2042:

- With UniProt facts and no MANE cross-reference, a request whose residue differs from the canonical protein still gets a silent isoform answer (`src/entities/variant/get.rs:570-572`). The Outcome names this case, and no test covers it.
- The build record defers the UniProt caching (latency 0.35 to 1.2 seconds, confirmed live) and the refusal text (it still prints `Candidates:` and never names the isoform that spells the request). The ticket says "Defers: nothing", and Ian's ruling makes both tag items.
- The "refuse a headline with no protein change" arm (`get.rs:576-588`) can return `Ok(None)` with every test green, since its only test reaches the past-the-end arm. A past-the-end refusal with no MANE transcript says the record carries no protein change on any transcript when it does.
- Nothing pins `snpeff.ann.feature_type` in the MyVariant field lists (`src/sources/myvariant.rs:24`, `:54`). Dropping it brings back the 32-row drop for Q1878R. H1883D has no test.

Gaps in 2043:

- Deleting the brand tier (`src/entities/drug/label.rs:801-809`) or passing the card name instead of the user's query at `src/entities/drug/get.rs:823` leaves every test green, because in every fixture the brand's label also wins another tier. A brand whose plain-ingredient twin is another labeler's product, such as Gleevec against generic imatinib, needs a test.
- `cached_discover_rescue` (`get.rs:381-391`) saves nothing on a refusal, since the first site returns at `get.rs:770`. The repeat lookup is the CLI alias fallback (`src/cli/shared.rs:440`, `no_cache: true`). No test covers the cache.
- `plain_paired_ingredient_name` (`label.rs:774-780`) splits any name at " and " without checking for hyaluronidase. Phesgo's ingredient line becomes "PERTUZUMAB, TRASTUZUMAB,", and a combination such as Opdualag could become a nivolumab card carrying the Opdualag label.
- The Proof asks for a recorded live sweep of 74 names; the record covers about 25. Rituxan and rituximab have only the live sweep.

Gaps in 2044:

- Merge `321969db0` took 2043's inventory reasons for `src/entities/drug/get.rs` and `src/transform/drug.rs`, erasing the history 2044 restored. The updater's growth arm overwrites the reason (`tools/update-rust-source-size-inventory:83`). Commit `c907cdd60` hand-edited seven inventory entries.
- The same merge dropped the explicit `.config` assertion from `tests/test_source_package_boundary.py`. The exclude survives in `Cargo.toml:14`.
- Replacing `hgvsp` or `consequence` with None in the JSON next-command arm (`src/cli/variant/dispatch.rs:452-453`), or the consequence at `:509`, passes every test.
- Still open from its Changes: lower the `src/transform/variant.rs` floor, move the test out of `src/render/json.rs`, and let the updater approve several files in one run.
- Remove the private-name scan. Ian ruled on 2026-10-10 that leaving other projects' names out of public repos is a writing habit that needs no guard and no GitHub secret. Delete the `PM_FORBIDDEN_NAMES` step in `.github/workflows/ci.yml` (around line 360), the forbidden-name declaration files and their reader in `tools/check-zero-coupling.py`, and the tests that cover them.

Gap in 2045:

- Every deadline test already runs inside a 60-second tokio watchdog (`deadline.rs:155-162` and siblings), so the new 180-second bound on the held reply cannot change an existing test's result. The observed hang sat outside the timed body. The red test is new and built to hang; it does not reproduce the observed hang, and a bare `cargo test` with the fix reverted still hangs.

- Starts from: main `c6d972d47`.
- Keeps: every fixed answer in the live sweep above; the generic label and numbering rules with no per-brand or per-gene lists.
- Changes: give article disease rows a link that opens the named disease, or the v0.9.1 search command; fix each live defect above; do the 2042 caching and refusal text; add the missing arm and tests; add a brand-tier test that fails when the tier is deleted; make the refusal path run the rescue once or drop the claim; scope the pairing split to hyaluronidase; restore the inventory history and the `.config` assertion; pin both hint flags in both arms; find and fix the 2045 hang's real location; finish the 2044 Changes items.
- Proof: for each item, a test that fails on `c6d972d47` or a mutation named in the record that the new test catches; a recorded live sweep of the 74 names and the variant set above on a build-host binary of the landing tree.
- Defers: nothing.
