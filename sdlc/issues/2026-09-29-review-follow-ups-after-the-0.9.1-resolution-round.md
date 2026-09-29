# Review follow-ups after the 0.9.1 resolution round

Filed 2026-09-29 from an independent read-only review of main at `c62b6594`, range `9eea4a86..c62b6594`. Three fresh reviewers covered the release and #287 fixes, the process changes, the GWAS and ClinGen fixes, the checks, the licensing records and both 2026-09-28 dispositions. Reviewers ran dry runs and mutations in scratch copies, read-only GitHub queries and public web fetches only.

The product fixes hold. The job order, the stdio test, #287's build profile, the asset smoke, the exit codes and `debug-embed` are confirmed. The ClinGen `--no-cache` test fails when the carry is reverted. Ian's direction from 2026-09-28 is not yet met: main did not stay green, neither disposition names a CI run, several deferrals lack Ian's OK, and some records claim fixes that did not happen. The CI run for `c62b6594` (36583345352) had not finished when this was filed, so "main is green at `c62b6594`" was unproven.

## Ian's direction, 2026-09-29

- Enrichr: relabel it as restricted, the way CGI is labeled. State its terms plainly, including the commercial-licence limit and the statement that it is not for treating or diagnosing human subjects. Enrichr stays on by default. BioMCP does not buy a licence or contact Mount Sinai.
- The same answer holds for every source from now on. BioMCP states each source's terms plainly in its registry and licensing page. Users are responsible for their own licences.
- Add a visible warning near the top of the README. It says that upstream terms govern the use of retrieved data, that some sources restrict commercial or clinical use, and that users must check the terms. It links to the Source Licensing and Terms page.
- The developer adds this rule to `AGENTS.md`, so every future source change follows it.
- Main stays unprotected. No branch protection.
- Ian cuts releases, and users install releases. Main is a development branch, and anyone who installs from main takes that risk. A red main is not a release blocker. The bar is the tagged commit: it must have a finished green CI run and pass the release workflow.
- Merging only after the branch's CI run is green stays the recommended practice. A red main costs the developer time and hides the next failure.

## Decisions for Ian

- Enrichr's terms are readable. The help page loads an empty `templates/help/terms-content.html`. The sibling `https://maayanlab.cloud/Enrichr/templates/help/terms-submenu.html` holds the terms in plain HTML. They say the tools are free for academic and non-profit use, and commercial use needs a licence from Mount Sinai Innovation Partners. They also say Enrichr is not to be used for treating or diagnosing human subjects. The registry calls Enrichr tier 1 (`docs/reference/sources.json`, `docs/reference/source-licensing.md:60`, `:352`). The commercial-licence limit is a tier 3 fact, as with CGI. Any choice that contacts Mount Sinai or buys a licence is Ian's.
- The deferrals below need Ian's OK under his 2026-09-28 direction. (Decided 2026-09-29: Ian approved the first three and refused the search-all deferral. See `2026-09-29-review-of-the-enrichr-cache-and-records-round.md`.)
  - The leftover check spellings, moved to the 1.0 compiler-tool replacement.
  - The marked waits, moved to the 1.0 signal-wait rewrite.
  - The four 1.0 issue files dated 2026-09-29. Each cites Ian's direction, and Ian has not confirmed.
  - The search-all end-to-end note test, deleted in `5dfb3845` and `2cf31950` and recorded only in a test comment.
- Main has no branch protection. The CI trigger on `tickets/**` records results and blocks nothing. Requiring the CI check before a push to main is a repository setting.

## Process

- Red commits still reached main after the new rule. `f4020068` reached main 33 seconds after its branch run started, and that run failed. `fda79d35` had no branch run. Both failed `test_pull_request_contracts_remain_separate_from_protected_release`. Main failed seven CI runs on 2026-09-29, from 06:13 to 12:47.
- Both dispositions say no commit went straight to main after `9eea4a86`. That is false. The ticket 1265 commits `a1caed47` through `b1393806`, `7eef70a0`, the fix-ups `5dfb3845` through `3d293ad7`, and `f4020068` through `c62b6594` are first-parent commits on main.
- The Rust cache runs before checkout in every job (`.github/workflows/ci.yml:30-31`). The logs show no lockfiles considered, a key suffix of `da39a3ee` (the hash of empty input), and the runner's Rust 1.98.1 in the key instead of the pinned 1.93.1. The key never changes with `Cargo.lock`, and checkout likely clears what was restored. Green run times went from 33-35 minutes to 26.8 minutes on the branch and 31.7 on main.
- The cache action uses the unpinned `@v2` tag, and `e18501b8` exempted it from the pin contract. Nextest installs through `curl | tar` with no checksum.
- Ticket 1275, which changed CI, says `Code review: n/a`.
- 62 remote `tickets/*` branches remain, 60 of them merged, including 1261 through 1276. The lander rule says the lander removes its branch.

## Release

- The changelog record pattern caps ticket numbers under 2000 (`scripts/check-changelog-coverage.py:18,21,66`). At ticket 2000, records drop silently and the gate fails open. Reject the date shape `^\d{4}-\d{2}-` instead.
- The gate reports 38 missing bullets at `c62b6594`, not 32: 1220, 1222-1224, 1226-1239, 1241-1244, 1247, 1248, 1250, 1254, 1255, 1257, 1259, 1261, 1263, 1264, 1266-1270 and 1275. Ticket 1253 still says 30. Tickets 1265 through 1269 have no records.
- The `--release --locked` assertion (`tests/test_release_workflow_provenance.py:693-736`) matches substrings in the whole step text, so a step with one release build and one plain build passes.
- The 0.8.x no-yank decision gives two false reasons. v0.8.25 built wheels with `maturin-action args: --release`, so the wheel sizes do not show a debug build. v0.8.22 already had `src/cli/skill`, `chart.rs` and rust-embed. Keep the decision and correct the reasons.
- Runbook step 8 says the release goes public after every publisher succeeds. The tap now runs after it. Ticket 1253's Ordering section does not mention the tap waiting for `publish-release`.

## Product and tests

- The GWAS behavior holds. `tests/test_gwas_no_store_contract.py` only scans source text. The stated reason for a text scan is false: reqwest-middleware 0.4.2 exposes `RequestBuilder::extensions()` publicly (`client.rs:572`). Replace the scan with a Rust test that builds a GWAS request under the infinite override and asserts NoStore. Ticket 1268 names `gwas_never_serves_stale_even_under_infinite_cache_mode`, which does not exist.
- The search-all end-to-end note test is gone. Changing `src/cli/outcome.rs:150` to pass `Vec::new()` fails no test. The disposition and ticket 1268 item 3 still claim a hermetic search-all test. Restore an end-to-end test with every search-all section served from local fixtures.
- `NoTrustFailureStrategy` (`src/sources/mod.rs:1150-1193`) matches error-text markers and has no test. Deleting its early return fails nothing.

## Checks

- The raise-review mechanism (`tools/check-test-wait-ratchet.py:271-300`) accepts any `Code review:` line with `ACCEPT` within 200 characters in any cited ticket. `Code review: pending; reviewer returns ACCEPT or findings` and `REJECT, not ACCEPT yet` both count. A raise can cite any old accepted ticket, and the author can write the line.
- `--update` still repins new waits in one command. Two sleeps added to gencc moved its pin from 4 to 5 and passed. A new file with waits also passed.
- The marker ceiling is 43, and it peaked at 46 during the round.
- Ticket 1264's Deferred gaps does not name container env, `options: -e BASH_ENV`, `with: BASH_FUNC_` or time bindings. The "stops here, accepted" decision appears only in ticket 1275 and not in the review file's disposition.
- Still passing: `REJECT, will be fixed later`, `not ACCEPTED`, `from time import monotonic`, `time.time()` polls and `checked_duration_since`. These belong with the check-spelling deferral.

## Licensing and records

- `docs/reference/sources.json:357` still says current CPIC URLs continue to resolve.
- `sources.json:1546` still says the WikiPathways licence is hosted on the classic site.
- The evidence file header still says 42/2/4 with three open (`docs/reference/source-licensing-evidence-2026-09-27.md:6-9`). The other four files say 45/2/1.
- Neither 2026-09-28 disposition names a CI run. They cite commits and yellow gate SHAs. The branch commits cited as proof (`f86a548e`, `346068be`, `e01a39f1`, `80095256`) have no CI runs.
- The 1255-1261 disposition has no line for the DDInter comment, the MyDisease.info branch, the stderr bound, the duplicate `sources.json` sentences, the COSMIC sentence, the GenCC stand-in, the yellow-overlap fragment, the DDInter issue file, 1258's Deferred gaps, the flake issue or the run history. Some were fixed earlier, in `2c573320` and `84920f31` among others.
- The 2026-09-26 disposition still says "the pending-review merge" (line 71), "the 1251 fix merge" (78), "the 1250 fix merge" (97), "1252/1248" (110), "(in flight)" (88, 114) and "1242 b1" (121). It is still hard-wrapped. The 1263/1264 disposition says it was rewritten.
- The 1263/1264 disposition says the hard-wrapped files were unwrapped in `c1b2ab0d`. These are still hard-wrapped: both new dispositions, the 1263/1264 addendum, both 1270 files, both 1275 files, all four 2026-09-29 issues, the pass record, the evidence file's open-items paragraph and the aspirin issue. Shorthand remains: "535747d9's chain", "P2s", "MECHANIZED", "round-three" and "~14 MB".
- Ticket 1260 still says "the variant JSON payload builder" and "the variant CLI module". Name `src/cli/variant/dispatch.rs`, `src/cli/variant/normalization_json.rs` and `src/transform/variant.rs`. Its prior evidence points at a local experiment that is not in the repo. Summarize the finding in the ticket.
