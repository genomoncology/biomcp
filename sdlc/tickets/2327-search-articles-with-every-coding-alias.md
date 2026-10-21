# 2327 — Search articles with every coding alias

Status: OPEN.

Milestone: 0.9.4

- Changes: `src/entities/article/variant_search.rs`, `tests/test_variant_article_alias_coverage.py`, `tests/fixtures/article/coding-aliases/authored.json`, `CHANGELOG.md`.

## Outcome

`biomcp variant articles` recovers the papers that each resolved coding spelling finds, not only discloses the spellings it left out. For ABCC8 R1437Q, the provider queries include c.4310G>A and c.4373G>A, or a follow-up query per dropped spelling merges its papers, within each provider's query length and rate limits. Disclosure stays for any spelling still left out, with its reason.

- Proof: an outside-in replay test for ABCC8 R1437Q shows both coding spellings in provider queries and their papers in the merged result, with complete=true when nothing is dropped; a test with more aliases than the limit still reports alias_limit and truncated=true.

## Evidence

- Historical D1 reproduction evidence is preserved on Yellow at `/home/ian/workspace/data/biomcp/ticket-evidence/2327/research-493-d1`, with sanitized provenance SHA256 `301d55e6966b24f800583f9a3775d0c58996e3401820bf7ef60a2280fb22b6fc`. The capture dates to 2026-10-19 and does not establish the 2026-10-21 recheck. Its `command/execution.json` records the default union command with `--debug-plan --verify-identity`. Neither omitted coding spelling was queried in that capture. The response bodies stay private and are not public fixture inputs. The replay fixture and its paper rows are authored controls.

- On 2026-10-21, the research team reran its logged errors on the 0.9.4 rehearsal binary cdb9f4a1b. Item D1, from 2246, still fails: article search for ABCC8 R1437Q leaves c.4310G>A and c.4373G>A out of provider queries. 0.9.4 discloses the gap (alias_limit, complete=false, truncated=true), but it does not recover the papers those spellings find. Research expects the searches to include these aliases.
- 2246 met its disclose-or-send outcome. This ticket adds recovery.
- Other-transcript forms still follow the transcript-alias decision.
