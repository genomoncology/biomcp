# Review follow-ups after the 1238 close

Filed 2026-09-27 from an independent read-only review of main at `79573b59`. Five fresh reviewers covered 1243, 1242 batch-1 fixes and batch 2, 1254 batches 2 and 3, the pending-review check, the 1250, 1246, 1251 and 1252 fix merges, and the 1238 close. Nothing blocks. The core work is real. The items below are gaps, overclaims and dropped findings.

## Ian's direction, 2026-09-27

- The developer runs the DDInter real-bundle checks on the local Mac host. The Mac-run issue no longer waits on Ian.
- The developer owns the source licensing review, including the 2027-03 review-date crossing and DepMap's terms. Research the sources, decide, and record the decision with citations. Bring Ian only a decision that spends money, signs an agreement, or contacts a data provider.

## Product

- `src/sources/vaers.rs:329`, `src/sources/hpa.rs:434`, `src/sources/medlineplus.rs:185` and `src/sources/pmc_oa.rs:227` call `Document::parse` directly and skip `parse_external_xml`. The 1243 depth cap misses these four upstream paths. roxmltree recurses once per open tag, so deep input can still overflow the stack. Route them through `parse_external_xml` and add a repo check that fails on a direct `Document::parse` outside `src/xml.rs`.
- The stale cache age reaches only a `warn!` line on stderr (`src/sources/mod.rs:203-223`). MCP and JSON consumers never see it. Ticket 1242 acceptance says each listed path shows the failure. Put the age in the output notes. The function that writes it has no test.
- `assign_top_genes` (`src/entities/disease/enrichment.rs:574-609`) knows only Monarch and CIViC as fallback sources. Genes seeded from MyDisease or DisGeNET are labeled Open Targets on the requested-sections path as well as the default card. The fix needs a real MyDisease label. No open issue tracks this. The source-failures issue is marked resolved and calls it the next pass.
- The search-all dropped-filter note pastes raw upstream error text (`src/entities/search_all/dispatch.rs:421`). Say plainly which source failed.
- `src/sources/ddinter.rs:471` still labels an HTML download reply as an unreadable bundle. This was filed on 2026-09-26 and dropped.

## Tests

- `the_mcp_path_builds_no_one_shot_runtime` in `src/cli/outcome/probe_tests.rs` reads the process-wide `ONE_SHOT_RUNTIMES_BUILT` counter. Parallel tests that call `run_outcome` bump it, so the test can fail at random. It also passes `WorkerDrive::Shared` by hand and never goes through `execute_mcp_cli`, so it proves nothing about the MCP path.
- `comments_cdata_and_self_closing_tags_do_not_move_the_depth` in `src/xml.rs` has no CDATA in its input and hides three opens against a cap of 64. Give it CDATA and more than 64 hidden opens.
- The release-mode panic test runs only in `make verify` (`Makefile:113`), the opt-in live lane. It needs no network. Put it in CI.
- `an_allowed_base_keeps_the_path_and_query` (`src/cli/health/http.rs:567`) passes only in debug builds. It and the test at `:553` set a process variable without a serial guard.
- `tests/tls_ca_bundle_contract.rs:604` drives two `biomcp version` calls. Confirm they build an HTTP client. If not, the test proves only the startup warning.
- GenCC still has no wrong-owner or not-a-directory cleanup test (filed 2026-09-26, dropped).

## Checks that still leak

Each check now catches the named cases, and a neighboring spelling still passes. Replace string bans with structure:

- **Pending-review check** (`tests/test_sdlc_review_status_contract.py`):
  - It reads one line, so a wrapped `re-review pending` passes.
  - It never flags a REJECT without a later ACCEPT. 1249 still has one.
  - Any scope containing "batch" or "item" exempts the line.
  - Bold, bullet-star and dash variants pass.
  - Tickets without a record are skipped, so 1191 through 1218 still say pending.
  - 1254:187 says "Code review (batch 3): pending" after its accept.
  - Require one fixed Review line format and fail anything else.
- **Workflow contract** (`tests/test_release_workflow_provenance.py:245-252`):
  - These pass: `|| echo skip`, `set +o errexit`, `trap -- 'exit 0' EXIT`, `|| exit $((0))`, and `defaults.run.shell: bash {0}` at job or workflow level.
  - `if: runner.os == 'Linux'` on the main smoke step turns off the Mac and Windows smoke and passes.
  - Pin the exact text of the smoke and gate steps, for example by hash, instead of banning phrases.
- **Wait ratchet** (`tools/check-test-wait-ratchet.py:60,106`):
  - `# watchdog: foo` passes, and marked lines are never counted. The tree has 25 markers.
  - These pass: `deadline < start.elapsed()`, `elapsed().as_millis() >`, `sleep_until(`, `anyio.sleep` and `import time as t; t.sleep`.
  - Cap the marker count.
- **Stdio guard** (`tests/test_source_child_stdio_guard.py:27`): `use std::process::{Command as Cmd}` passes with stdin removed from the icacls call.
- **Schema merge** (`src/mcp/shell.rs:391-402`): a constraint present on one side only, such as `uniqueItems` or `maxLength`, is copied silently and narrows the root. ADR 0002 still says "wider in accepted values".

## Earlier review file

The previous review file, `sdlc/issues/2026-09-26-review-follow-ups-for-1242-through-1254.md`, now carries its own item-by-item disposition section (added with this file's follow-ups); its four dropped findings are accounted for there.

## Records

- The 1238 record says every open issue file has a decision. Four have neither a Resolved nor a Decision section:
  - the DDInter Mac run;
  - the one-CPU deadlock;
  - `2026-09-26-review-follow-ups-for-1242-through-1254.md`;
  - spec-contracts, which has only a Verification section.
- `2026-09-26-review-follow-ups-for-1242-through-1254.md` is linked from nowhere and has no resolution. Most items were fixed. Four were dropped: the DDInter HTML label, the unrecorded brand-name change, the GenCC tests, and the 1244 release-prep items missing from 1253.
- Ticket 1254 contradicts itself:
  - `:111` says items 5-12 are not started.
  - `:187` says batch 3 review is pending.
  - `:188` and `:253` are stray merge text.
  - The record frontmatter still says `head: d8475987`.
- Ticket 1242 `:172` says the yellow gate has not run. The record says it passed at `bb516645`.
- Ticket 1243 has a stale "Unverified here" paragraph and cites old `manager.rs` line numbers.
- The 1254 record says credentials can never be redirected in release builds. A release build honors the override when both test variables point at loopback. State that.
- Lifecycle-test failures are called pre-existing in the spec-contracts issue at line 142. No issue tracks them.
- Ticket 1253 says 24 changelog bullets are missing. A dry run reports 26, adding 1238 and 1243. Its after-tag list omits closing GitHub #284 after 0.9.1 reaches PyPI.
- `2026-09-24-yellow-build-overlaps-another-repository-gate.md:3` and `2026-09-25-test-server-terminated-without-confirmed-ownership.md:3` say "Status: open" above a Resolved section.
- The yellow-overlap Resolved section says BioMCP gate scripts take the lock. The lock code lives in dotfiles, and dotfiles has an open issue showing the lock free during BioMCP checks.
- `2026-09-11-variant-display-mixes-transcript-numbering.md:11` parks a real clinical-report failure to 1.0 with no owner or trigger. The other 1.0 candidates also lack an owner and trigger. Give each a ticket or a revisit trigger.
- Decision text uses shorthand a new reader cannot follow: "the 1252 regime", "the marked-watchdog form", "the 1242 render walk".

## Disposition (2026-09-27, tickets 1255-1261)

Every item in this file, marked. Tickets name their merge; the gates ran green at the SHAs in each ticket's Review section.

Product — all fixed: the four XML sources route through the depth cap with a structural guard on any direct parse (ticket 1255, merged 9148e874, gate 8bf9884a); the stale-cache age reaches the MCP card, the CLI card, and search JSON `_meta.notes`, and MyDisease/DisGeNET genes carry their real labels on both paths (ticket 1256, rebased landing at 16952792); the search-all note names the failed source in plain words and the DDInter HTML reply is a download failure (both in ticket 1256's landing); the DDInter HTML label from the 2026-09-26 file closed with it.

Tests — all fixed: the shared-runtime test drives `execute_mcp_cli` with its own armed-window counter (1257, gate a9dee294); the CDATA test carries real CDATA and 65 hidden opens per family; the release-mode panic test is a pinned CI job; the health tests pass in release honestly with serial guards; the TLS test's evidence is now the dial-and-refuse proof; the GenCC wrong-owner and not-a-directory tests exist.

Checks — rebuilt structurally (1258, gate 3f6916db): one review grammar with scope-vocabulary matching and backfilled legacy verdicts; 19 workflow steps hash-pinned; the wait ratchet counts markers against a ceiling and resolves aliased waits; the stdio guard resolves import aliases; merge_property panics on one-sided constraints; ADR 0002 states the precise rule.

Records — fixed through ticket 1259's commits: both review files carry dispositions (this section and the 2026-09-26 file's); the four no-decision files have Decision or Resolved sections; the ticket contradictions, frontmatter heads, credentials claim, Status lines, yellow-lock wording, 1253 counts (26 plus the #284 close), 1.0 owners and triggers (variant transcripts → ticket 1260), and the shorthand rewrites are all on main.

Direction items — done: the Mac DDInter run executed on the local Mac host at a7d503be with all three checks passing (its issue file carries the run and numbers); the licensing review pass covered all 48 sources due 2026-03-20 (45 verified after the 2026-09-28 corrections, COSMIC and CPIC materially changed and updated, Enrichr unverifiable and kept dated, record `sdlc/records/2026-09-27-source-licensing-review-pass.md`); DepMap decided under the delegation with citations (stays out; the 2026-09-19 note's reservation superseded by the recorded decision); the 2027-03 crossing handled for 47 of 48 — Enrichr fails 2027-03-21 unless its terms page is fixed and verified first.

Beyond this file: ticket 1261 fixed the cache-mode leak from GitHub #286 (Ian's filing; close #286 at the release).
