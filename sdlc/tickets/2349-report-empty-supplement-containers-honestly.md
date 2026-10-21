# 2349 — Report empty supplement containers honestly

Status: OPEN.

Milestone: 0.9.4

## Outcome

When a supplement route returns HTTP 200 with an empty XML container, BioMCP reports that the source listed no supplements, not that none exist, and names any other route the user can try.

- Proof: outside-in replay tests with an empty container show the in-band 'source listed none' status and wording; a populated container is unchanged.

## Evidence

- On 2026-10-21, research reran five hereditary cancer variants with blind agents on qualified BioMCP 0.9.4, with no search cap and direct provider calls allowed. Evidence: research repo commit 561cb47, sdlc/records/0041-tool-issues.tsv (27 observations with exact commands) and 0041-direct-only-findings.tsv (56 direct-only traces). These are observations, not verified defects; confirm on current main first and withdraw with evidence if it does not reproduce.
- Item 8: supplement routes report empty XML containers with HTTP 200. Neither proves that no supplements exist.

- Coordinator restatement: distinguish source-listed-none from unavailable sources and universal absence, using the existing browser recovery route. Current-main M5 j4166 replay on ff4e1c04e: `biomcp --json get article 22663014 assets` with recorded PMC7992806 HTTP200 201-byte empty namespaced fullTextXMLBean returned CLI1/source_unavailable and Europe PMC could not confirm asset availability. Wrapper0 captures observed failure, not productpass. Earlier j4161 failed recipe setup before CLI and is not product proof. Approved exact-container parser, asset observation/navigation and SourceListedNone enum changes preserve arbitrary XML failures and populated ZIP behavior; outside-in regression precedes edits. Captured XML/recipe retained on Yellow ticket-evidence/2349.
