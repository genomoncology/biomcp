# 2368 — Full-text warnings name the URL and status

Status: OPEN.

Milestone: 0.9.4

Priority: 1

- Changes: `CHANGELOG.md`, `src/entities/article/assets.rs`, `src/entities/article/fulltext.rs`, `src/entities/article/fulltext_budget.rs`, `src/sources/fulltext_diagnostics.rs`, `src/sources/mod.rs`, `src/sources/ordinary_url_policy.rs`, `src/sources/provider_url_policy.rs`, `tests/test_article_fulltext_warnings.py`.

## Outcome

Every full-text route warning names the request it made (URL without key values) and the HTTP status or transport error.

## Evidence

- Research recheck of the 0.9.4 candidate b56138d27 (research repo f1c305c, sdlc/records/0042-test-the-0-9-4-candidate-hard.md; raw outputs on Blue under /home/ian/workspace/data/research/runs/0042/), relayed by Jarvis in m3902. These are observations: reproduce on current main first, and withdraw with evidence if it does not reproduce.
- `biomcp --json get article 33766116 fulltext --lines 15:22` warns on stderr that the PMC OA asset manifest request failed, with no URL or HTTP status.

## Proof

- Outside-in test: a failed manifest fixture produces a warning with the URL and status, and no key value appears.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks this item on the final build.
