# Rebuild the phrase-banning checks as structural checks

From the 2026-09-27 review file ("Checks that still leak") and
Ian's feedback point 2: every current check bans exact phrases, and
the next spelling over still passes. Structure closes the whole
class. Each rebuilt check must catch a spelling it was never told
about — prove that with a mutation the check was not built from.

## Problem

1. **Pending-review check** (`tests/test_sdlc_review_status_contract.py`)
   reads one line (wrapped `re-review pending` passes), never flags
   a REJECT without a later ACCEPT (1249 has one), exempts any scope
   containing "batch" or "item", accepts bold/bullet variants, and
   skips tickets without records (1191-1218 still say pending).
   Ticket 1254:187 says "Code review (batch 3): pending" after its
   accept.
2. **Workflow contract** (`tests/test_release_workflow_provenance.py:245-252`)
   still lets `|| echo skip`, `set +o errexit`, `trap -- 'exit 0' EXIT`,
   `|| exit $((0))`, and job/workflow-level
   `defaults.run.shell: bash {0}` through; an `if: runner.os == 'Linux'`
   on the main smoke step turns off Mac and Windows smoke and passes.
3. **Wait ratchet** (`tools/check-test-wait-ratchet.py:60,106`):
   `# watchdog: foo` passes and marked lines are never counted (25
   markers in tree); `deadline < start.elapsed()`,
   `elapsed().as_millis() >`, `sleep_until(`, `anyio.sleep`, and
   `import time as t; t.sleep` all pass.
4. **Stdio guard** (`tests/test_source_child_stdio_guard.py:27`):
   `use std::process::{Command as Cmd}` passes with stdin removed
   from the icacls call.
5. **Schema merge** (`src/mcp/shell.rs:391-402`): a constraint
   present on one side only (`uniqueItems`, `maxLength`, ...) is
   copied silently and narrows the root; ADR 0002 still says "wider
   in accepted values".

## Fix

1. **One Review grammar.** Every ticket file must carry Review
   lines in one exact format (design, code, verification), however
   wrapped; the check joins wrapped lines before matching, so no
   wrapped spelling passes. A REJECT verdict anywhere requires a
   later ACCEPT in the same file. Scoped lines (batches, items) are
   allowed only while the later state in the file leaves them
   genuinely open, and the scope token must match the ticket's own
   batch/item vocabulary — not any line containing the word.
   Verdict-less legacy tickets 1191-1218 get honest backfilled
   verdict lines (from their records and git history; where history
   is unknown, say "record not kept at the time" rather than
   inventing). Fix 1254:187 while in there.
2. **Hash-pin the load-bearing steps.** For the smoke and gate
   steps the contract cares about, pin the exact step text: extract
   each step from the parsed YAML, normalize trivial whitespace, and
   compare a SHA-256 recorded in the test. Drift fails with the
   diff. The mutation tests then prove the pin: any edit to a pinned
   step, including `if:` conditions, shells, and continue-on-error
   removals, changes the hash and fails. Keep a short comment block
   in the workflow naming which steps are pinned so an editor knows
   where to update the hash. Ban `defaults.run.shell` at job and
   workflow level structurally (any `shell:` outside a step fails).
3. **Count the markers.** The wait ratchet records the marker count
   (25 today) with a ceiling in its inventory JSON; a new marker
   requires raising the ceiling with a reason, in the same commit.
   Marked lines are counted and reported. Add the missed forms:
   any `elapsed()`-comparison wait, `sleep_until(`, `anyio.sleep`,
   and aliased/imported `time` sleeps (catch `import time as X` and
   `from time import sleep as X` by resolving the local name).
4. **Resolve the alias.** The stdio guard parses `use` lines, maps
   local aliases (`Command as Cmd`), and then checks every call
   site of the resolved name for the three-stream shape; unknown
   spellings fail rather than pass.
5. **No silent narrowing.** `merge_property` treats one-sided
   constraint keywords (`uniqueItems`, `maxLength`, `minLength`,
   `pattern`, `format`, `multipleOf`, etc.) as clashes: panic with
   the keyword named, add should-panic tests, and fix any real
   collision the tripwire finds by resolving it explicitly (union
   semantics or a deliberate choice recorded in ADR 0002). Correct
   ADR 0002's "wider in accepted values" to the precise rule the
   code enforces.

## Review

- Design review: n/a (Ian's feedback specified the structure)
- Code review: BLOCK once (dead ruff-breaking block, fmt indent,
  missing scope-vocabulary check), folded and verified 2026-09-27;
  report-only P2s (REJECTED inflection, prose rejections,
  sleep_ms, *_tests.rs sidecars, PINNED_STEPS completeness) are
  candidates for the next pass
- Verification: yellow gate at 3f6916db — lint, test, spec, stress
  OK, zero failed lines

## Deferred gaps (recorded 2026-09-28; closed by ticket 1264)

The review named these report-only holes in the first cut, plus two
new ones it proved. Ticket 1264 (this branch) closes them:

- A new step slipped before a pinned step, writing BASH_ENV or
  swapping the binary, kept every pinned hash clean. Fixed by the
  exact step-list pins: job id, order, count.
- BASH_ENV in any env block (job, workflow, step) or named in a
  `with:` value reinterprets every bash run invisibly. Banned
  everywhere.
- A duplicated needs line in a pinned job: the YAML parser keeps
  the last, the text scan read the first. Both the parsed needs and
  a one-line-per-job count now enforce the expected list.
- The verdict grammar was a phrase net, not an allowlist:
  "awaiting reviewer", Security/Ticket review kinds, headings,
  paragraphs, "(batch final)", "REJECT, not yet fixed", bare
  wrapped state words, and landed tickets with no Review lines all
  passed. The whole-line allowlist plus scope-word, negation, and
  no-record rules close them.
- The wait ratchet missed reversed and `<=` deadline comparisons,
  `Instant::now() <` polls, Rust use-aliases, and assigned Python
  time modules.
- The stdio guard exempted `.output()` wholesale and had a dead
  fail-closed branch; unknown stream settings now fail closed.
- P2s from the first review that remain open in 1264's scope
  check: none — all closed here.

## Acceptance

- Each rebuilt check has at least one mutation test proving it
  catches a spelling not in its original ban list (name it).
- The verdict-grammar check passes on the fixed tree, flags the
  unresolved-rejection red run recorded in 1249, and covers every
  ticket file including the backfilled 1191-1218.
- The workflow contract fails on a one-character edit to any pinned
  step (prove one), and on each leak spelling from the problem list.
- The wait ratchet reports the marker count and fails when the
  ceiling is exceeded; each missed form is proven red on a planted
  file.
- The schema merge panics on one-sided `uniqueItems` and
  `maxLength` (should-panic tests), and ADR 0002 matches the code.
- Yellow gate green.
