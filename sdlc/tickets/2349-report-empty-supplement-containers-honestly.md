# 2349 — Report empty supplement containers honestly

Status: OPEN.

Milestone: 0.9.4

Reviews: revision ec88f75df9c9965f2cf13a54019d1c80378cc5dc, accept

## Outcome

When a supplement route returns HTTP 200 with an empty XML container, BioMCP reports that the source listed no supplements, not that none exist, and names any other route the user can try.

- Proof: outside-in replay tests with an empty container show the in-band 'source listed none' status and wording; a populated container is unchanged.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 8: supplement routes report empty XML containers with HTTP 200. Neither proves that no supplements exist.

- Changes: `src/sources/europepmc.rs`, `src/entities/article/assets.rs`, `src/entities/article/mod.rs`, `tests/article_asset_terminal/source_outcomes.rs`, `spec/fixtures/setup-article-fulltext-source-fixture.sh`, `testdata/sources/europepmc/pmc7992806-empty-supplementary.xml`, `CHANGELOG.md`. Recognize the recorded empty source container and report its limited meaning with an existing article route.
- Coordinator restatement: distinguish source-listed-none from unavailable sources and universal absence, using the existing browser recovery route. Current-main M5 j4166 replay on ff4e1c04e: `biomcp --json get article 22663014 assets` with recorded PMC7992806 HTTP200 201-byte empty namespaced fullTextXMLBean returned CLI1/source_unavailable and Europe PMC could not confirm asset availability. Wrapper0 captures observed failure, not productpass. Earlier j4161 failed recipe setup before CLI and is not product proof. Approved exact-container parser, asset observation/navigation and SourceListedNone enum changes preserve arbitrary XML failures and populated ZIP behavior; outside-in regression precedes edits. Captured XML/recipe retained on Yellow ticket-evidence/2349.

- Actual regression RED j4174 on ff4e1c04e failed the scoped-empty CLI assertion (nextest100; wrapper0 explicitly expected RED). Final affected j4203 passed seven CLI source-outcome cases, including scoped absence/browser route, unknown optional fulltext counts and populated ZIP control, with all-target Clippy, formatting and shell syntax. Unchanged parser cases passed ten in j4185. j4196 used an authored wrong PDF control; corrected control uses existing successful HTML, not a product-policy change. Exact final source transport patch SHA256 d6ca0cbfc57fe1623548a0d9bb3f70b5168eb6a34f629acb79f6ca6b59136dfa; final formatted code/test diff SHA256 4c213428cc08f57915b1e48954c3b876ad6ff8b00e5fa7438d7984c2fdb6e4cf. Raw logs, recipes, patches and captured 201-byte XML retained on Yellow ticket-evidence/2349.
- Fresh independent reviewer accepted ec88f75df9c9965f2cf13a54019d1c80378cc5dc after full diff/caller review and independent exact-published-source M5 j4210 CLI pass. Its code/test diff matches the final j4203 patch exactly. No reviewer changes. Batch landing gate remains required.

- Batch gate j4244 failed at source-capture lint because the new XML lacked its inventory receipt. The owning developer verified the committed 201-byte XML against the HTTP 200 raw capture and its recorded request-start timestamp. The candidate adds only that required receipt row, without changing fixture bytes or production behavior; the failed gate remains failed and the corrected candidate requires the declared gate.
