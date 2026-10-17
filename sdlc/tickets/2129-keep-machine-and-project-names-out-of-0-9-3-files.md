---
priority: 3
---

# 2129 — Remove private names from the release changes

Status: OPEN.

Milestone: 0.9.3

Depends on: 2120
Depends on: 2095
Depends on: 2152

## Outcome

A human review of additions since v0.9.2 removes private build-machine and other-project names while preserving meaningful provider identifiers and historical technical evidence.

- Starts from: the added-line diff since v0.9.2, including the baseline record and capture receipts. The measurement script already uses job scheduling and has no host lock to replace.
- Keeps: source identifiers such as `SIG-M5`, factual measurements, public BioMCP names and text unchanged since v0.9.2. Do not add a banned-word check or a new host-lock setting.
- Changes: claims `sdlc/planning/milestones.md` `sdlc/pm.json` `sdlc/records/2041-merge-main-into-1-0-build.md` `sdlc/records/2069-0-9-3-baseline.md` `sdlc/records/2078-article-html-stack-build.md` `sdlc/records/2079-replay-server-build.md` `sdlc/records/2113-civic-one-letter-build.md` `sdlc/records/2124-disable-sources-everywhere-build.md` `sdlc/records/2125-close-the-release-gate-holes-build.md` `sdlc/records/2126-article-search-deadline-build.md` `sdlc/records/2127-bring-back-spec-checks-build.md` `sdlc/records/2130-keep-the-pubtator-variant-entity-an-rsid-search-finds-build.md` `sdlc/records/2132-move-main-to-the-next-dev-version-build.md` `sdlc/records/2133-stop-every-command-walking-the-whole-http-cache-build.md` `sdlc/records/2137-0-9-gates-on-the-macos-gate-host-build.md` `sdlc/records/2143-zip-symlink-advisory-build.md` `sdlc/records/2144-exact-name-strategy-and-yield-per-alias-build.md` `sdlc/records/2145-mark-an-empty-article-search-incomplete-when-a-source-is-down-build.md` `sdlc/records/2146-refuse-an-empty-supplement-download-build.md` `sdlc/records/2147-report-every-section-batch-article-is-asked-for-build.md` `sdlc/records/2150-find-a-non-empty-copy-of-every-europe-pmc-supplement-build.md` `sdlc/tickets/2061-disable-sources-by-name-or-group.md` `sdlc/tickets/2064-fix-the-install-paths-the-0-9-2-test-found.md` `sdlc/tickets/2070-delete-build-rs-and-build-once-per-ticket.md` `sdlc/tickets/2085-refuse-a-drug-name-several-products-share.md` `sdlc/tickets/2086-skip-full-ci-for-ticket-record-pushes.md` `sdlc/tickets/2089-run-search-alls-two-gwas-queries-at-the-same-time.md` `sdlc/tickets/2092-run-the-start-up-cache-clean-at-most-once-a-day.md` `sdlc/tickets/2093-bump-rustls-past-the-fresh-advisory.md` `sdlc/tickets/2098-let-the-1-0-gate-host-run-all-three-gates.md` `sdlc/tickets/2102-bound-the-article-deadline-contract-test-under-load.md` `sdlc/tickets/2103-bound-the-remaining-load-sensitive-deadline-tests.md` `sdlc/tickets/2106-find-the-one-failing-rust-test-on-the-m5-gate-host.md` `sdlc/tickets/2107-fix-the-fourteen-macos-only-spec-page-failures-on-the-1-0-line.md` `sdlc/tickets/2126-find-why-an-instant-article-search-spends-its-whole-deadline.md` `testdata/sources/capture-receipts.json` `CHANGELOG.md`. Review these specific files and replace private names with the role or explanation a public reader needs. Add any newly discovered file to this claim before editing it.
- Proof: inspect `git diff v0.9.2 HEAD` additions manually after the other measurement-script writers land; show the reviewed replacements preserve the meaning and provider identifiers. Run only existing affected text contracts through the build job. No new enforcement mechanism is required.
- Defers: a historical repository-wide rewrite.

PM decision: 2026-10-17. This is a human public-text cleanup; preserve valid provider data. Complete it after the script and coverage changes, before final measurement.
