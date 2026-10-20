---
flow: build
priority: 3
---

# Degraded and capped card sections print an executable recovery command

## Outcome

When a Markdown card says that a section is degraded, unavailable, or cut off
with more rows known, it also prints the exact read-only BioMCP command that
retries that work or retrieves the next rows. The command is generated from
the same typed card identity, section registry, source, and pagination state as
the output; it is not hand-built in a template.

## Current facts and reproducer

This ticket came from
`sdlc/issues/2026-08-27-degraded-and-capped-sections-should-print-their-recovery-command.md`.
The current tree at `e1179c4f` has since closed two of the three reported
shapes:

- `src/cli/search_all/dispatch.rs::dispatch_section` already attaches a
  `search.retry` link containing the direct canonical search command to every
  failed or timed-out `search all` section. `templates/search_all.md.j2`
  prints it, and the same link remains structured in JSON.
- A capped disease diagnostics card already prints the shell-quoted command
  built by `disease_diagnostic_search_command`. The command opens the broader
  paged `search diagnostic --source all --limit 50` result set. Keep this as a
  regression contract; do not replace it with an offset-only fragment.

Three actionable gaps remain.

The successful article-search card has a second, non-`SectionOutcomes` status
path. `article_source_status_note` prints degraded/unavailable
`ArticleSourceStatus` rows (for example `Europe PMC source status: degraded
(Europe PMC timed out after 12s)`) but has no access to an executable
source-specific retry. The JSON response retains these rows in
`_meta.source_status`, while `_meta.next_commands` omits the retry. A hard
article-search error that returns no card is not this shape; this ticket does
not convert command failure into a partial card.

The article template currently places that status block only in its
non-empty-results branch. Consequently, a successful zero-row response can
retain degraded statuses in JSON while Markdown says only that no articles
matched. A successful zero-row card is still in scope: it must render the
existing typed degraded/unavailable statuses and any valid retries. This does
not change search success/failure classification.

First, `section_render_contexts` in `src/render/markdown/mod.rs` turns typed
`SectionOutcome::degraded` and `SectionOutcome::unavailable` values into honest
status prose, but it receives no card identity and emits no recovery command.
That shared path feeds article, diagnostic, disease, drug, gene, pathway, PGx,
protein, and variant Markdown. For example, the existing unavailable-CIViC
fixture in `src/render/markdown/gene/tests/rendering.rs` renders:

```text
**CIViC status (CIViC):** unavailable; no conclusion can be drawn — CIViC gene evidence is unavailable.
```

and stops. `variant_structure_markdown` has the same gap on its separate
`lookup_outcomes` path for InterPro and Cancer Hotspots.

Second, trial location cards still disclose continuation without providing
one. `trial_markdown` truncates an unpaginated `locations`/`all` Markdown card
at 20 and prints `Locations: showing 20 of 21 (display cap 20).` An explicit
page from `src/cli/trial/dispatch.rs` prints `more available`, and JSON exposes
only `{total, offset, limit, has_more}`. None supplies the next command even
though `get trial --offset --limit ... locations` is supported.

The package file ceiling is already saturated: `cargo package --list
--allow-dirty --locked --offline --no-verify | wc -l` reports exactly **1300**.
No new tracked file may be added.

## Required behavior

### Failed source-backed detail sections

- A displayed `degraded` or `unavailable` section status is immediately
  followed by exactly one `Retry:` line containing one backticked command.
  `Data`, `empty`, `not_requested`, and `inapplicable` states do not gain a
  retry.
- Canonical registered sections retry only that section:
  `biomcp get <entity> <shell-quoted-id> <canonical-section>`.
  Use the canonical target in `SELECTOR_ROWS`; never print an alias.
- Outcome-only states may retry the smallest existing callable owner, but only
  through an explicit central route mapping. In particular, a source fact
  shown on a base disease/variant card may retry that card, and the separate
  structure outcome retries `biomcp variant structure <id>`. Do not fabricate
  section tokens or source flags that the parser does not support.
- The card identity must be the identity already used for its existing `More`
  commands: PMID/PMCID/DOI fallback for articles, accession for diagnostics and
  proteins, canonical ID for diseases/pathways, query for PGx, name for drugs,
  symbol for genes, and `preferred_variant_follow_up_id` for variants.
- Build commands as argument vectors with `crate::next_command::NextCommand`
  (or an equally central typed helper) and render each argument through the
  existing shell quoting owner. Provider text, status messages, and templates
  must never be interpolated into shell commands.
- Wrap rendered commands with a Markdown code-span helper whose delimiter is
  longer than any backtick run in the command. Shell escaping a literal
  backtick does not by itself stop that character from closing a Markdown code
  span. The visible code-span content must remain the exact shell command that
  `shlex` parses; do not replace or drop identifier/filter characters to make
  Markdown easier to render.
- `variant_structure_markdown` uses its input variant and adds the same one
  exact structure retry after either recoverable lookup failure; if both
  lookups fail, print it once.
- The registry/recovery helper is the single policy seam. Add a deterministic
  invariant covering every source-state row that is rendered as a recoverable
  status, so a newly registered recoverable section cannot silently lack a
  callable route. Exceptional outcome-only routes must be enumerated in that
  seam, not scattered among templates.

### Degraded article-search sources

- A successful federated article-search card with a degraded or unavailable
  source prints one direct retry per affected planner-compatible source, using
  an explicit `ArticleSource`-to-flag mapping (`pubtator`, `europepmc`,
  `pubmed`, `semanticscholar`, or `litsense2`), never the provider display
  label. This applies when the successful card has zero rows as well as when it
  has results.
- Build the retry from `ArticleSearchFilters`, the failed `ArticleSource`, and
  the current limit/offset. Preserve every semantically applicable public
  query, date, article-type, journal, access/retraction, sort, and ranking
  option. Omit federation-only `--max-per-source` on a direct-source retry.
  Use canonical flags and the shared argument-vector quoting owner rather than
  rebuilding the human query summary.
- Validate the candidate with the existing article planner before printing it.
  If that direct source cannot accept the selected filters, omit that command;
  do not offer a retry that immediately fails validation. Preserve the typed
  degraded status even when no direct retry is valid.
- Put the same commands in JSON `_meta.next_commands`; keep the existing
  `_meta.source_status` entries unchanged. Markdown prints each command next
  to its source-status line rather than burying it among unrelated pivots.

The retry is an affordance, not a success guarantee: a still-unhealthy remote
provider may fail again. It must parse, select the intended section/helper, be
MCP-allowlisted as read-only, and make the request when invoked.

### Capped and paged trial locations

- When generic `trial_markdown` applies its 20-location display cap, append a
  `Next:` line containing `biomcp get trial <id> [--source nci] --offset 20
  --limit 20 [contacts] locations` as one backticked command. Include
  `contacts` only when the current requested
  view includes contacts (`all` counts as including contacts), so site-contact
  alignment is preserved. Omit `--source ctgov`, the default; preserve
  `--source nci` for NCI cards.
- Determine the source from typed dispatch state when it is available, or from
  an exact mapping of the entity's normalized provider markers
  (`ClinicalTrials.gov` and `NCI CTS`). Do not feed those display markers to
  `TrialSource::from_flag`, and do not silently treat an unknown non-empty
  source as CTGov.
- For an explicit location page with `has_more`, compute the next offset with
  checked/saturating arithmetic as `offset + returned`, preserve the current
  page limit, trial source, and whether contacts were requested, and print the
  full command after the pagination disclosure. Do not derive the next offset
  from `total`, and do not emit a command when `has_more` is false.
- Add the same optional `continuation_command` to JSON
  `location_pagination`. Existing keys and their meanings are unchanged. The
  command belongs in pagination metadata, not in a location row. Omit the key
  when there is no continuation rather than serializing it as `null`.
- Reuse one trial-location command builder for generic Markdown, explicit-page
  Markdown, and JSON so their commands cannot drift.

### Surface compatibility

- Ordinary CLI Markdown and Markdown returned through the MCP `shell` tool
  contain the recovery line. Every generated command must pass both Clap
  parsing and the MCP read-only allowlist.
- Detail JSON retains its typed `section_outcomes`, provenance, and existing
  `_meta.next_commands`; this ticket does not replace typed state with prose.
  Prepend/dedupe the same applicable recovery commands in
  `_meta.next_commands`, including a requested section that failed and would
  otherwise be absent from progressive-disclosure follow-ups. Existing
  detail-card follow-ups remain present. Trial location JSON gains only the
  additive optional pagination continuation described above.
- Existing `search all` error links, disease-diagnostics recovery, PGx section
  continuations, discover continuation, drug-region continuation, and article
  asset continuation remain unchanged.

## Test-first acceptance

1. Add failing renderer tests before implementation for a canonical degraded
   section and unavailable section. Use at least one adversarial identifier
   containing whitespace, a literal backtick, and other shell metacharacters.
   Assert an intact Markdown code span with exact command text, one occurrence,
   ordering immediately after the status, and absence for
   healthy/inapplicable states.
2. Extend the existing printed-command round-trip contract (do not add a new
   test file) so generated recovery commands are split with `shlex`, accepted
   by `try_parse_cli`, and select the intended canonical route. Cover a normal
   `get ... <section>` command, an outcome-only route, and `variant structure`.
   Teach the extractor to recognize the variable-length code-span delimiter;
   weakening the adversarial fixture to suit the current single-backtick split
   is not acceptable.
   Add the MCP read-only assertion in the existing `src/mcp/shell.rs` test
   module, where `is_allowed_mcp_command` is visible.
3. Add an exhaustive registry test proving every recoverable rendered
   `SOURCE_STATE_ROWS` entry resolves to exactly one route. It must fail for a
   synthetic unmapped row/route or equivalent negative seam, preventing a new
   source-state section from shipping as prose-only.
   Extend the existing detail-card Markdown/JSON surface-agreement test with
   degraded fixtures so the same retry is present in JSON `_meta.next_commands`.
4. Extend existing trial renderer/dispatch tests to prove all of these exact
   cases: 20 of 21 generic `all` rows produces offset 20/limit 20 with
   `contacts locations`; exactly 20 rows has no continuation; an explicit
   offset 20/limit 3 page with more rows produces offset 23/limit 3; both the
   `ClinicalTrials.gov` and `NCI CTS` entity markers map to the intended CLI
   source behavior; an NCI page retains `--source nci`; the terminal page omits
   the JSON key and emits no Markdown command; and adversarial IDs are one
   safely quoted argument.
5. Add article-search renderer/JSON tests for two degraded sources with
   adversarial multiword/metacharacter filters. Assert stable source flags,
   preserved applicable filters/offset/limit, omission of federation-only
   options, no command for healthy or planner-incompatible sources, identical
   Markdown/JSON command sets, Clap parsing, and MCP allowlisting. Include a
   successful zero-result card and prove its status/retry are visible in
   Markdown rather than only in JSON.
6. Extend the existing trial mustmatch fixture in `spec/entity/trial.md` and
   its existing fixture script (no new packaged files) to execute the printed
   CTGov continuation command as printed and prove that the next expected
   facility rows are returned. Assert the additive JSON
   `location_pagination.continuation_command` on the same receipted fixture.
7. Keep the already-working `search all` failure link and disease-diagnostics
   command under focused regression assertions. No live network is required.
8. Run `make lint`, `make test`, and `make spec`. Finally verify the package
   list remains exactly 1300 and run `git diff --check`.

## Scope and non-goals

In scope are recoverable source-state failures rendered inside detail cards,
degraded source rows on a successful article-search card, the separate
variant-structure lookup states, the known disease-diagnostics cap regression,
and trial-location section continuation.

This ticket does not change upstream timeouts, retry middleware, source
selection, section caps, ranking, result completeness, partial-result policy,
or failure classification. A command-level failure that produces no card is
outside this section-rendering ticket. It does not promise that a retry
succeeds. It does not treat summary shortening, table-cell previews, bounded
error messages, provider traversal safety ceilings, or a complete terminal
page as a continuable section. It does not invent a command for an
inapplicable state or for data that no public command can retrieve.

## Likely implementation surfaces and rails

- Shared recovery policy/formatting:
  `src/render/markdown/mod.rs`, `src/render/markdown/sections.rs`,
  `src/entities/source_state_registry.rs`, and existing colocated tests.
- Identity plumbing into the shared renderer: the existing Markdown modules
  for article, diagnostic, disease, drug, gene, pathway, PGx, protein, and
  variant. Templates should not need per-section command logic.
- Separate structure path: `src/render/markdown/variant.rs` and its existing
  tests.
- Trial continuation: `src/render/markdown/trial.rs`,
  `src/cli/trial/dispatch.rs`, `src/cli/trial/tests_locations.rs`, existing
  trial renderer tests, `spec/entity/trial.md`, and its existing fixture
  helper only if fixture output needs adjustment.
- Cross-surface command contract: `src/cli/tests/printed_card_commands.rs` and
  `src/cli/tests/surface_agreement.rs` plus existing MCP/parser test modules.
  Article-search retry construction also touches the existing article
  renderer/dispatch and JSON tests. Reuse existing helpers instead of creating
  a sidecar.
- Documentation only where the observable contract is already described:
  `docs/user-guide/trial.md` and `docs/troubleshooting.md` if their wording
  needs the exact new command.

Do not add a tracked file or dependency. Keep every touched Rust source at or
below 1000 lines unless it already has a pinned inventory baseline, never
increase a pinned over-limit baseline, keep every `src/cli` Rust file at or
below 700 lines, and keep `src/entities/section_outcome.rs` below its separate
700-line policy cap. `src/cli/trial/dispatch.rs` is 575 lines,
`src/cli/trial/tests_locations.rs` is 602, `src/render/markdown/mod.rs` is 782,
`src/render/markdown/sections.rs` is 364, and
`src/entities/source_state_registry.rs` is 757 at design time.
`src/cli/article/dispatch.rs` is already 689 lines against the 700-line CLI
cap. Split or compact existing modules/tests rather than crossing a rail.
`src/render/markdown/article/tests.rs` and `src/mcp/shell.rs` are already above
the generic limit under pinned inventories and must not grow in net lines;
`src/render/markdown/variant/tests.rs` is at 995 and cannot absorb a normal new
test without compaction or moving coverage to an existing smaller test module.

## Dependencies

Ticket 1141 already established location/contact alignment; preserve it. The
current source-state registry, typed outcomes, central shell quoting, MCP
allowlist, and existing pagination commands are the implementation seams. No
provider change, live credential, new fixture, or other ticket is required.

## Design review

**ACCEPT as amended (2026-09-04).** The three shapes share one observable
recovery-command contract and can use central typed builders without changing
provider behavior or public schemas beyond the stated optional JSON field.
The amendments make zero-result status visibility, Markdown delimiter safety,
trial source recovery, terminal pagination shape, and saturated line/file
rails explicit. Hard command-level failures remain excluded because they do
not produce the partial card this ticket owns; existing `search all` and
disease-diagnostics commands remain regression-only.

## Implementation evidence

Implemented at `e1179c4f` on 2026-09-04. Recovery routing now has one typed
registry seam shared by detail Markdown and JSON; structure lookup failures
deduplicate to one structure retry. Article retries are built from the typed
filter/page state and admitted only after direct-source planner validation.
Trial continuation uses one typed builder for generic Markdown, explicit-page
Markdown, and optional JSON pagination metadata. Commands use the central
argument-vector renderer, and Markdown uses a delimiter longer than any
backtick run in the rendered command.

The test-first red run selected the new gene and generic-trial contracts: 0 of
2 passed. The gene card emitted no retry, and a 21-location trial card emitted
no continuation. After implementation, the focused recovery suite passed 13
of 13 tests. `make lint`, `make test`, and `make spec` pass; `make test`
reported 3144 Rust tests passed with 30 skipped and 892 Python contracts passed
with 3 skipped, followed by a successful strict documentation build. The
package list remains exactly 1300 files, `git diff --check` passes, all touched
CLI files remain at or below 700 lines, and the three pinned files remain at
their exact ceilings (`article/tests.rs` 1231, `variant/tests.rs` 995, and
`mcp/shell.rs` 2136). No tracked file or dependency was added.

Independent code review found two omissions: explicit `all locations` pages
did not retain contacts in their continuation, and recovery routes carried the
internal `adverse_event` registry key into a CLI that exposes
`adverse-event`. The remediation red run passed 0 of 3 focused tests: the
`all` continuation was absent, the exhaustive generated-command parser
rejected `adverse_event`, and the MCP allowlist rejected the same command.
Recovery routes now own the callable CLI entity token, and exhaustive tests
parse and MCP-check a generated command for every source-state row. Explicit
pagination treats `all` as a contact-bearing view. The same three focused
tests then passed 3 of 3, and the full 13-test ticket selection remained
green. The post-remediation gates passed: `make lint`; `make test` with 3144
Rust tests passed and 30 skipped plus 892 Python contracts passed and 3
skipped, followed by the strict documentation build; and `make spec`.

## Independent re-review

**ACCEPT with no remaining findings.** The original reviewer verified that
explicit `all locations` continuation preserves contacts in the canonical
`contacts locations` order and that recovery routes own callable CLI entity
tokens, including `adverse_event` to `adverse-event`. Every registry row's
generated recovery command is Markdown-extracted, Clap-parsed, and
MCP-allowlisted. Seven independent focused checks, package and line-count
rails, and `git diff --check` passed; the 41-file diff remains within the
 accepted ticket scope with no new file or dependency.

## Completed 2026-09-04

Degraded and unavailable detail sections now provide parser-valid, shell-safe
recovery commands consistently in Markdown and JSON; variant structure
deduplicates its recovery action. Successful federated article cards expose
planner-valid direct-source retries even when no rows are returned. Generic and
explicit trial-location pages now expose exact continuation commands and
additive JSON pagination metadata while preserving source selection, contact
alignment, filters, and offset semantics; terminal pages omit the continuation
field. Generated routes are exhaustively Clap-parsed and MCP-allowlisted.

Primary verification passed after independent re-review: `make lint`; `make
test` (3,144 Rust tests passed with 30 skipped, 892 Python tests passed with 3
skipped, and strict documentation passed); and `make spec` (all routine groups,
including 140 serialized cases with 4 skipped, 39 parallel-isolation cases,
and 8 static cases). Packaging remains exactly 1,300 files, all touched line
rails and pinned baselines hold, and `git diff --check` passes.
