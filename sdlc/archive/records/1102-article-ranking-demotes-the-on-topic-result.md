---
flow: build
priority: 3
---

# Make hybrid article ranking use lexical coverage and disclose a partial top match

## Outcome

Keyword-bearing `search article` results use the fraction of query anchors found
in each title/abstract as the hybrid lexical component. A highly cited one-term
match therefore cannot receive the same lexical score as a five-term match. If
the top relevance-ranked row still does not contain every keyword anchor, both
Markdown and compact/full JSON say that its lexical coverage is partial instead
of presenting it as an unqualified answer.

## Current facts and reproducer

The 2026-09-01 repository build (`biomcp 0.9.0-dev.6`) produced:

```text
$ biomcp search article -k "zolgensma treatment of retinoblastoma randomized trial" --limit 3
| Identifier | Title | Why | Cit. |
| PMID 39013849 | Exploring treatment options in cancer: Tumor treatment strat… | hybrid 0.325 + title+abstract 2/6 | 741 |
| PMID 32328653 | Glioblastoma in adults: a Society for Neuro-Oncology (SNO) a… | hybrid 0.3 + title+abstract 1/6 | 982 |
| PMID 26427984 | Conservative treatment of retinoblastoma: a prospective phas… | hybrid 0.261 + title 5/6 | 25 |
```

The query deliberately combines a spinal-muscular-atrophy drug with an
unrelated eye tumour, so live provider order is evidence of the failure shape,
not a stable test fixture. The same failure class is recorded in
`sdlc/issues/2026-08-27-article-keyword-search-has-no-relevance-floor.md`.

The owning implementation is now `src/entities/article/ranking.rs` (the
original ticket predates the article-module split). `lexical_ranking_metadata`
already records `anchor_count`, `title_anchor_hits`, and
`combined_anchor_hits`. However, `rank_articles_hybrid` currently calculates
`lexical_score` as `directness_tier / 3`. Every partial match has tier 1, so a
1-of-6 and a 5-of-6 row both receive `0.333`; citations and source position can
then put the one-term row first. The Markdown `Why` column displays the richer
hit counts, but they do not affect the hybrid score.

Article search already has a public warning envelope in `src/cli/article/mod.rs`:
date sorting emits an in-band Markdown warning and a structured JSON
`_meta.warnings[]` entry. Compact JSON deliberately omits row-level `ranking`,
so a warning must use that envelope rather than relying on `--full` diagnostics.

## Behavior contract

- In hybrid mode, `lexical_score` is
  `combined_anchor_hits / anchor_count`, with `0.0` when `anchor_count == 0`.
  `combined_anchor_hits` is the union of anchors found in the title or abstract:
  an anchor found in both counts once, not twice. Derive the ratio from the full
  counts before the public `u8` display counters are saturated. The score
  remains finite and in `[0, 1]`. Do not change the four default hybrid weights
  or the citation, semantic, and source-position normalization.
- Lexical mode keeps its existing tiered comparator and PubMed-rescue policy.
  This ticket changes the numeric lexical component of hybrid mode, not the
  meaning of `directness_tier` or the standalone lexical order. PubMed-rescue
  metadata must not become an implicit hybrid bonus, and an exact hybrid-score
  tie keeps the existing stable-identifier order.
- For a keyword-bearing relevance search with at least one returned row, emit a
  warning when the first row's ranking metadata has `anchor_count > 0` and
  `combined_anchor_hits < anchor_count`. Use the stable code
  `partial_query_match` and wording that says the **top result has partial
  lexical query coverage** and directs the reader to the `Why` column or full
  ranking metadata before citing it. Do not claim the row is biologically
  irrelevant: semantic matches and synonyms can be valid without literal
  overlap. For this warning policy, "partial" means any incomplete literal
  coverage, including zero hits; exact coverage does not warn. The policy is
  the same for explicit lexical, semantic, and hybrid relevance modes.
- The warning appears before the table in Markdown and in
  `_meta.warnings[]` for both compact and `--full` JSON. It composes with the
  existing warning representation; date/citation sorts and entity-only
  relevance searches do not gain this keyword-coverage warning.
- Keep per-row `Why` text and full JSON ranking diagnostics. After the scoring
  change their displayed/stored lexical and composite scores must agree with
  the actual order.
- Do not silently filter rows or invent a numeric semantic relevance floor.
  With heterogeneous providers, literal coverage is evidence that can be
  stated exactly; it is not proof that a synonym-aware semantic result is bad.

## Test-first acceptance

1. Add a synthetic hybrid-ranking regression in
   `src/entities/article/ranking/tests/calibration/hybrid.rs` before changing
   production code. Use two otherwise comparable zero-semantic candidates for
   the six-anchor keyword query: one matches five anchors with 25 citations and
   one matches one anchor with 982 citations. The test must initially reproduce
   the cited row winning, then prove the five-anchor row ranks first and the
   stored lexical scores are `5.0 / 6.0` and `1.0 / 6.0`. Assert the composite
   order rather than hard-coding live provider scores.
2. Preserve the existing worked-example, custom-weight, semantic-source-gating,
   zero-safe, PubMed-rescue, and lexical-order tests. Add focused assertions
   that a title-and-abstract duplicate counts once, that a zero-anchor hybrid
   calculation is finite `0.0`, and that an exact hybrid-score tie retains the
   stable-identifier order. Adjust an existing expected value only if the new
   coverage formula directly explains it; do not refresh unrelated ordering
   wholesale.
3. Add focused warning-policy tests for all trigger boundaries: partial first
   row warns; a zero-hit first row warns without being called irrelevant; a
   fully covered first row does not; no results do not; an entity-only relevance
   search does not; and non-relevance sorts do not. Cover an explicit
   non-hybrid relevance mode so the warning is demonstrably independent of the
   selected relevance comparator.
4. Prove the public output contract with deterministic renderer/CLI tests:
   Markdown contains `Warning:` plus the partial-coverage message, and compact
   and full JSON contain `_meta.warnings[]` with code
   `partial_query_match`. Compact JSON must remain compact (no row-level
   `ranking` field).
5. Run the focused Rust tests for article ranking and article CLI/renderer
   output, then the repository gates exactly:

   ```bash
   make lint
   make test
   make spec
   ```

## Scope and likely files

Expected production/test scope:

- `src/entities/article/ranking.rs`
- `src/entities/article/ranking/tests/calibration/hybrid.rs`
- `src/cli/article/mod.rs`
- `src/cli/article/dispatch.rs`
- existing tests under `src/cli/article/tests/` and
  `src/render/markdown/article/tests.rs` as needed
- `docs/how-to/find-articles.md` for the scoring and warning contract

Use existing files. `cargo package --list --allow-dirty --locked --offline
--no-verify` is already at the 1,300-file ceiling, so do not add a packaged
file. Rust production sources must stay below the enforced 1,000-line ratchet;
at design time `ranking.rs` is 546 lines, `cli/article/mod.rs` 389, and
`cli/article/dispatch.rs` 688. The existing renderer test file is already over
1,000 lines under its recorded allowance; avoid growing it if the same public
contract can be proved in a smaller existing article test module.

## Exclusions and dependencies

- No provider, federation membership, fetch-depth, deduplication, source-cap,
  pagination, retraction, or enrichment changes.
- No changes to explicit `--sort date` or `--sort citations` ordering.
- No changes to standalone lexical or semantic ranking, PubMed rescue, default
  weights, or the weight-override CLI.
- No live-upstream order assertion and no new relevance threshold or stop-word
  system.
- Trial search and `variant articles` ranking are separate contracts. The
  partial-query warning is for direct keyword-bearing `search article` output;
  do not broaden it to other entity renderers without separate evidence.
- No prerequisite ticket is required. Current baseline is `e2017afe`, with
  ticket 1101's gnomAD change already landed; it does not overlap this scope.

## Design recommendation

**ACCEPT.** The defect is reproducible directly from the current scoring code,
the necessary hit counts already exist, and the response already has a warning
channel. The revised scope replaces the original ambiguous request for a
"relevance floor" with deterministic proportional scoring plus an exact,
non-clinical warning about partial lexical coverage.

## Independent design review

**ACCEPT (as amended).** Review against `e2017afe` confirmed the owning score,
existing lexical/PubMed-rescue boundaries, hybrid weights and deterministic
identifier tie-break, warning envelope, compact/full JSON distinction, and
package/file-size rails. The acceptance plan now also fixes the previously
implicit union-count, zero-anchor, zero-hit-warning, explicit-mode, and hybrid
tie boundaries.

## Implementation evidence

- Test-first red: seven focused tests ran against the old implementation; four
  failed as intended because the 982-citation one-anchor row ranked first, the
  duplicate title/abstract fixture stored the tier-derived `2/3` score instead
  of `1.0`, and compact/full JSON emitted no partial-coverage warning. The
  zero-safe, stable-tie, and non-trigger checks remained green.
- Green: the new and directly affected article ranking, CLI, and renderer suite
  passes 125/125. This includes exact `5/6` versus `1/6` stored scores, union
  counting, zero-anchor finiteness, stable identifier ties, full-count scoring
  before `u8` display saturation, all warning boundaries, Markdown placement,
  compact JSON without row ranking, and full JSON with ranking diagnostics.
- Repository gates: `make lint`, `make test`, and `make spec` pass. `make test`
  reports 3,138 Rust tests passed (30 skipped), 892 Python tests passed (3
  skipped), and a clean strict docs build. The routine spec groups, 39 parallel
  isolation tests, and 8 static checks pass.
- Packaging remains exactly 1,300 files. Production sources remain below 1,000
  lines (`ranking.rs` 552, `cli/article/mod.rs` 418, `dispatch.rs` 689); the new
  CLI tests live in the existing filter test module, below its
  700-line ratchet.

## Independent code review

**REJECT with one correctness finding.** The warning policy compared the
saturated public `u8` hit counters, so a partial 260-of-300 match stored both
`anchor_count` and `combined_anchor_hits` as 255 and incorrectly emitted no
warning. The owning ranker already preserves the exact unsaturated result in
`all_anchors_in_text`.

## Remediation evidence

- A compact-JSON regression with saturated 255/255 public counters and exact
  `all_anchors_in_text = false` failed before remediation because
  `_meta.warnings[]` was absent.
- The warning predicate now keeps the nonzero-anchor guard and uses
  `!all_anchors_in_text`. The regression passes, as do all 125 focused article
  ranking, CLI, and renderer tests.
- After remediation, `make lint`, `make test`, and `make spec` pass with the
  counts recorded above. Packaging remains exactly 1,300 files.

## Independent re-review

**ACCEPT with no remaining findings.** The original reviewer verified that the
nonzero-anchor guard now uses the exact unsaturated `all_anchors_in_text`
signal, and that the compact-JSON 260-of-300 regression emits
`partial_query_match` despite both public counters saturating at 255. The 13
hybrid calibration tests, five warning/output boundary tests, package and
source-size rails, and `git diff --check` all pass; the diff remains limited to
the seven accepted ticket files.

## Completed 2026-09-04

Hybrid article ranking now derives its lexical component from the exact union
of title/abstract query-anchor hits divided by the full anchor count, while
preserving standalone lexical ordering, PubMed rescue, default and custom
weights, other normalized components, and stable identifier ties. Direct
keyword relevance output emits the structured `partial_query_match` warning in
Markdown and compact/full JSON whenever the top row lacks exact full literal
coverage, including when public hit counters saturate.

Primary verification passed after independent re-review: `make lint`; `make
test` (3,138 Rust tests passed with 30 skipped, 892 Python tests passed with 3
skipped, and strict documentation passed); and `make spec` (all routine groups,
including 140 serialized cases with 4 skipped, 39 parallel-isolation cases,
and 8 static cases). Packaging remains exactly 1,300 files and `git diff
--check` passes.
