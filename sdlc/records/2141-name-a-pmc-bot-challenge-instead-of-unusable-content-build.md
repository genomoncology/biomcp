# 2141 build record — name a PMC bot challenge instead of calling it unusable content

Built: 2026-10-17. Branch `ticket/2141-name-a-pmc-bot-challenge-instead-of-unusable-content` from `origin/main` at `b138f6097`, rebased onto `origin/main` at `f9a11e81e` (landing of 2138; one CHANGELOG bullet conflict resolved by keeping both bullets, and the coupling digest repinned against the merged capture-receipts file). Post-rebase tip `1df0ddadd`, pushed. Stopped without landing; the coordinator lands.

## Reproduction on current main (before the fix)

Recorded in the ticket's Evidence section:

- Live, the reported 0.9.2 binary, the ticket's exact command twice: both runs returned `full_text` from PMC HTML — the gate is intermittent tonight and served the article both times.
- Deterministic, current main code at `6b4545ac3` (main + test-only): serving the recorded-shape challenge page through the PMC HTML step records `"reason": "unusable_content"`. Gate job `1792207045377-82291` (the macOS gate host), command `cargo nextest run --locked --no-default-features --lib pmc_bot_challenge`: 1 failed — `left: String("unusable_content"), right: "bot_challenge"`. The reported defect reproduces on main.

## What changed

- `html_is_bot_challenge` in `src/sources/pmc_article.rs` recognizes the challenge page by the two markers its recorded title carries ("checking your browser" and "recaptcha", both required, case-insensitive), so an article that merely mentions either word never matches. `fetch_html_with_client` returns a new `PmcHtmlFetchOutcome::BotChallenge` instead of article data when the page matches; the carry-all `html()` helper folds it into its existing error path.
- `src/entities/article/fulltext.rs` carries the new outcome through `FulltextStepOutcome::BotChallenge`; the PMC HTML step records the attempt with the new `ArticleFulltextAttemptReason::BotChallenge` (wire name `bot_challenge`), coverage `unusable`, outcome `unavailable`, and the same failure fold as before — the ladder keeps running (the PDF step still executes, unchanged), and the final section outcome keeps its previous semantics.
- `ArticleFulltextAttemptReason::BotChallenge` in `src/entities/article/mod.rs` (snake_case serialization `bot_challenge`); the reason enumeration in `spec/entity/article.md` carries the new name.
- Fixture `testdata/sources/pmc_article/pmc-bot-challenge.html` is authored provider-shaped from the recorded probe evidence (page title "Checking your browser - reCAPTCHA"); `capture-receipts.json` unions it in as `authored` with its reason, and the coupling digest in `tools/zero-coupling-historical.json` repins for that file change. No new file enters the shipped source package (testdata is package-excluded; the PDF the test reads was already packaged), so the package file pin stays at its own count.
- CHANGELOG carries the 0.9.3 Fixes bullet.

## Claims held by tests

- `entities::article::fulltext::tests::pmc_bot_challenge_page_names_its_reason_and_keeps_the_pdf_ladder_running` (recorded-reply fixture test; failed red on main at `6b4545ac3`): the challenge page yields reason `bot_challenge` on the wire and as the enum, coverage `unusable`, outcome `unavailable`; the PDF attempt still runs after it and becomes the winner.
- `sources::pmc_article::tests::bot_challenge_markers_need_both_recorded_title_words`: detection matches the recorded page and a differently-cased title, and rejects article text carrying only one marker.

## Gates

- Gate host (macOS) via the pm job turn-taking, at the pre-rebase tip `cb1a589ce`: `make lint` passed (job `1792207632319-36413`) after two honest fixes — one rustfmt pass over the test (`0b601ce35`) and the coupling-digest repin (`cb1a589ce`); earlier lint failures at `5c2ffaaba`/`0b601ce35` were the same two causes. Targeted red run `1792207045377-82291` (failed as claimed) and green run `1792207292859-6480` (2/2 passed) also ran there.
- Gate host (macOS) at the post-rebase tip `1df0ddadd`: `make lint` passed (job `1792211888455-11043`). `make spec` failed on exactly the two clingen-erepo frozen-capture blocks (job `1792217510701-14573`; 59 passed, 2 failed) — the identical two blocks fail `make spec` at `origin/main` `f9a11e81e` on the same host (job `1792217568586-17257`), so the failure is ambient to that page on that host, not this branch; the same spec lane is green in CI at this tip.
- Linux gate host at the post-rebase tip `1df0ddadd`: `make test` passed (job `1792211969750-2480127`) — full archived Rust suite, 1160 Python contracts passed (3 skipped), strict docs build clean; the package file pin and capture-receipt checks inside it passed unchanged. `make test` also passed at `cb1a589ce` (job `1792207697029-2220730`) before the rebase.
- CI (ci.yml) green at the exact post-rebase tip `1df0ddadd`: run `39003295189` (also green pre-rebase at `cb1a589ce`, run `38998826957`).

## Findings

- The challenge page family the probe recorded differs from the proof-of-work interstitial the linked-asset route already names (`pmc_proof_of_work`/`ncbi_interstitial`): the article-page gate carries the reCAPTCHA title, not the PoW markers. Detection keeps the two families separate; naming both remains one concept per spelling.
- The bundle installs a 0.9.2 binary; the live probe from this session reproduced the ticket's good path (full text from PMC HTML), not the gate. The deterministic replay is the standing reproduction.

## Cleanup

- /tmp scratch from this session (live-run JSON and stderr captures) deleted. The worktree and branch stay for the coordinator; no experiments created; nothing landed.
