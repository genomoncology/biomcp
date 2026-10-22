# 2354 — Read the returned fulltext conversion

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision b1f22f6fef106e77b7df8ead8d6292a1f5f1c1dd, accept

Landed: 4a93161

## Outcome

After BioMCP returns an existing managed fulltext conversion for an article, the documented immediate line-read commands can read that same conversion when a later provider attempt is unavailable. Check the actual file and persisted identity before repairing resolution. Preserve explicit cache controls and honest absence when no usable conversion exists.

- Changes: `src/entities/article/fulltext.rs`, `src/entities/article/fulltext_saved.rs`, `src/entities/article/mod.rs`, `src/cli/article/dispatch.rs`, `src/utils/download.rs`, `tests/article_asset_terminal.rs`, `tests/article_asset_terminal/saved_fulltext.rs`, and `CHANGELOG.md`. Persist the successful managed conversion locator and consume it for line and outline views. Preserve normal retrieval and explicit cache bypasses.
- Proof: reproduce article10811118 initial retrieval followed by `--lines 1:60` and `--lines 61:100` in one isolated cache, retaining stat/read proof before each call; an outside-in replay succeeds after later provider unavailability and refuses absent/unusable local content.

## Evidence

- PM m3492 authorizes uncovered confirmed defects from every research observation. Research c001/call-0032 (TSV physical line6) is not the two-PMID sequence tested by withdrawn2347. Those earlier j4148 non-reproductions remain correct historical evidence. pm refuses reopening a withdrawn ticket and directs creation of a fresh OPEN ticket; 2347 remains withdrawn.
- Current-main M5 j4335 on f7165ac3b6e2c25367f31f25393343313564b5e7: `biomcp --json get article 10811118 fulltext` exited0 and returned a managed text path. Immediate `biomcp --json get article 10811118 fulltext --lines 1:60` and then `--lines 61:100`, same home/cache, both exited1/not_found after renewed provider work. The first probe did not stat the file before each read; the owning lane must establish that exact boundary before choosing a fix. Yellow ticket-evidence/2350/followup-probe retains actual outputs and commands.
- Coordinator restatement: make documented reads consume the available returned conversion, rather than allowing a new unavailable provider response to erase that capability. No hypothetical fix before local-file/identity verification.

- Owning-lane current-main proof j4356 on f7165ac verifies the critical boundary: initial10811118 HTML conversion is65243 bytes/336 lines, regular0600, readable before BOTH line calls at the same inode and SHA256321eaa0a3e9af92c910c75a34206c6ba7d5e94d9b52eb03cc4cf98fca3671dab. Both documented1:60/61:100 calls still exit1/not_found. No caller or cache-directory drift.
- Actual recorded CLI RED j4364 on f7165ac plus test-only transport: initial fixture HTML retrieval and real local-file read succeed; redirect its later origin to an existing404 fixture, then the documented line call fails not_found (nextest100, wrapper explicitly expects RED). This is not a repaired success.
- Bounded implementation stores the successful managed conversion's winner and requested/resolved identity aliases in the existing SQLite fulltext namespace. Line/outline views consult this locator before provider resolution, derive only managed download paths and require usable UTF8; normal retrieval freshness and provider ladder stay intact. Explicit no-cache/cache-off bypass it. Source files and accurate claim name the new local helper, CLI callsite and limited managed-path visibility.
- Final affected j4369 passed all37 article_asset_terminal cases, including saved views after later provider absence, aliases, explicit bypass, empty/invalidUTF8/missing-file controls, plus Clippy with warnings denied and formatting. The coherent final branch and archive must bind its exact tested patch before acceptance. No routine/full-gate duplicate is needed; the batch lander owns that check.

- Fresh reviewer reproduced two additional boundaries on c3606d4: j4383 showed optional locator SQLite failure discarding a successful conversion; j4387 showed supported normalized PMID00022663012 missing the locator. The reviewer fixed best-effort persistence and reused the existing article identifier parser for lookup normalization, then strengthened actual CLI alias, outline and symlink controls.
- Reviewer accepts with fixes at b1f22f6fef106e77b7df8ead8d6292a1f5f1c1dd. Affected j4388 passed both saved-view outside-in tests, including aliases, explicit bypass, absent/unusable files, optional storage failure and secure-read refusal controls, plus Clippy. Earlier j4369's37 affected cases remain identified as their actual pre-review source proof. Yellow ticket-evidence/2354/2354-review-proof.tar.gz SHA2561a4ce14887d1bed2a41f47d2c2c6f9e07b89a02c468b38b4d81243c71a4932b0 retains reviewer evidence. Developer raw j4356/j4364/j4369 logs and tested formatted source artifacts remain in that same evidence directory. Batch landing gate remains required; no full suite duplicated.
