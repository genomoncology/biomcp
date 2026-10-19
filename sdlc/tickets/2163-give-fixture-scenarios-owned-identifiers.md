---
priority: 3
---

# 2163 — Give fixture scenarios owned identifiers

Status: OPEN.

Milestone: 0.9.4

## Outcome

Recorded article fixture scenarios have named owners and identifiers, so a fixture repair cannot silently change an unrelated scenario's provider replies.

- Starts from: the article fixtures and test consumers involved in the 2131/2150 regressions; inventory actual shared identifiers before choosing a migration batch.
- Keeps: source facts, provider behavior, product identity rules and existing assertion strength.
- Changes: `spec/fixtures/setup-article-fulltext-source-fixture.sh` `CHANGELOG.md` and this ticket for the demonstrated collision. No Rust consumer changes or shared scenario framework are needed for this batch.
- Proof: a recorded outside-in case demonstrates the inventoried collision before migration, then unrelated named scenarios continue to return their own replies when one is changed. Run affected cases through the build job.
- Defers: implementation until the next release. A narrow fixture repair needed for a current-release product fix stays in that product ticket; this ticket does not block 0.9.3.

PM decision: 2026-10-17. Ian deferred the broad scenario refactor to keep current-release prevention fixes small.

## Progress

- 2026-10-19 started

## Current collision and development brief

Restated outcome: isolate the independent citation forced-failure scenario from the resolver-order scenario so changing one reply cannot change the other scenario's behavior.

Current-main study `2163/1792413080397-15076` used fixture source `fefca03d4be1e16f6e143697ccabb6e0f4cfe902`, equivalent to main `9e9bc75b5acabf38ae9852119368bc29efcb9602`, with frozen CLI SHA256 `c0b04d420095ea47f21b0751f7241d7031522022db1c933f21c13c6da142887e`, banner `0.9.4-dev+g9e9bc75b`. No build ran. Resolver PMID22663014 obtains PMC123459 through ID Converter; independent citation PMID40001002 obtains the same synthetic PMCID through Semantic Scholar. Both request `/PMC123459/fullTextXML` from one fixture server.

Observable commands: `biomcp --no-cache --json get article 22663014 fulltext` and `biomcp --no-cache --json article citation-evidence 40001002 10.1099/unresolved-fixture --fulltext`. In an isolated fixture copy, replacing only that shared reply from404 to200 with the existing JATS payload changes resolver outcome empty to data, and also changes the independent citation from fulltext_unavailable with no locator to reference_unresolved from europe_pmc_jats at PMC123459. The full driver is retained in the job. Initial temporary-path preparation failures supplied no CLI result; the successful proof canonicalized its temporary root and cleaned its servers/copies.

Smallest fix: give the citation forced-failure scenario a named dedicated synthetic PMCID and explicit missing-JATS reply. Keep the resolver route and all other replies unchanged. Use the same isolated reply-mutation probe as outside-in RED/GREEN ownership proof, and retain existing article specification assertions for both independent outcomes and context, terminal XML/PDF refusal behavior, and MCP batch/redaction/typed-PDF behavior. No new framework or duplicated product test is needed.

The inventory found the older supplement/PDF identifiers already separate. Repeated identifiers in separate fixture instances do not establish a collision. Neither colliding scenario has a consumer in rmcp_client_contract.rs, so do not edit it. Record any remaining demonstrated collision before expanding this batch.

