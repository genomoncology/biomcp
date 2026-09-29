# Review follow-ups after tickets 1255 through 1261

Filed 2026-09-28 from an independent read-only review of main at `d5bcdd69`. Five fresh reviewers covered 1255, 1256, 1257, 1258, 1259, 1260, 1261, the stdio flake fixes, the licensing pass, the Mac DDInter run, and the release request. Reviewers ran local Python checks and read-only GitHub queries only. The product work is real. Two items block the release: the changelog gate bug and the missing #287 entry.

## Blocks the release

- `sdlc/records/2026-09-27-source-licensing-review-pass.md` breaks the changelog gate. `RECORD_TICKET` in `scripts/check-changelog-coverage.py:13` reads the leading year as ticket 2026. A dry run over `v0.9.0..origin/main` reports 31 missing entries: 26, then 1255, 1257, 1259 and 1261, then the fake 2026. Rename the record to the owning ticket's form or tighten the pattern, and add a test.
- GitHub #287 was filed 2026-09-28 by an outside user. It reports more fallout from the 0.9.0 debug-profile wheel: the skill and chart assets and the `--json` workflow ladders fail. Ticket 1253 does not list it, and it does not list #286. Confirm 0.9.1 fixes each symptom #287 names, and add both issues to the after-release close list.

## Release path

- `pypi-publish` in `.github/workflows/release.yml` waits for the wheel build, the wheel smoke and docs-live. It does not wait for the tarball or container jobs. The Homebrew tap update runs before PyPI. A container failure after the PyPI upload uses up 0.9.1. Ticket 1253 says PyPI comes before the tap, and the workflow does not enforce it. Make the tap and PyPI jobs wait for every build and smoke job.
- No path exercises the wheel, tarball and container jobs without publishing. A manual dispatch runs only the version check, docs and the container job. Add a dry-run input that builds and smoke-tests everything and uploads nothing, then rehearse it once on a branch before the tag.

## Product

- Three search-JSON outputs drop the stale-cache note. Only `search_meta_with_workflow` (`src/cli/shared/search_payloads.rs:185`) collects it. Article search JSON (`src/cli/article/mod.rs:187`), GWAS search JSON (`src/cli/gwas/dispatch.rs:54`) and search-all JSON build their own meta without it. The get-JSON issue says search JSON is covered. Fix the three paths or extend that issue.
- Notes are stored per task. A fetch started with `tokio::spawn` does not inherit them. The ClinGen prefetch in `get gene` (`src/entities/gene.rs:2519`) runs that way, so a stale ClinGen result shows no cache note. Audit every spawned fetch.
- The comment at `src/sources/ddinter.rs:66` says the echoed content type is ours. It comes from the upstream header (`:478`). Fix the comment, or stop echoing the header.
- The plain "MyDisease.info" branch in `src/entities/disease/enrichment.rs:608-617` has no producer. Remove it or give it a real input.

## Tests

- The stdio flake has a likely cause, and the 45-second budget does not address it. Both failed runs (36359461611, 36364074552) ended near 5.02 seconds, the same under the old 10-second limit and the new budget. The read loop ended on stdout closing. The test closes stdin at `tests/tls_ca_bundle_contract.rs:646` right after writing its requests. On stdin close, rmcp 1.7.0 gives in-flight replies 5 seconds and then drops them (`rmcp-1.7.0/src/service.rs:1052-1080`). Keep stdin open until the replies arrive. Remove the "CI-proven" comment at `:656`. File an issue for the flake with an owner and a trigger.
- A fetch that fails its TLS handshake takes about 2 seconds even in a passing run. Check whether the client retries an error that retrying cannot fix.
- The stderr diagnostic at `tests/tls_ca_bundle_contract.rs:689` reads to end with no timeout. Bound it the way `:579` does.
- `the_mcp_path_builds_no_one_shot_runtime` still asserts on the process-wide `ONE_SHOT_RUNTIMES_BUILT`. Nextest isolates it. Plain `cargo test` can still flake.

## Checks

- Workflow contract. These pass with all 81 tests green:
  - `BASH_ENV` set at job or workflow level, pointing at a file with `trap 'exit 0' EXIT`;
  - a new step before a pinned step that writes `BASH_ENV` to `$GITHUB_ENV` or swaps the installed binary;
  - a duplicate `needs:` line in `pypi-publish`, since the YAML parser keeps the last and the text check reads the first.
  Require each pinned job's step list to match exactly, and ban `BASH_ENV` in every `env:` block.
- Review grammar. These pass on a landed ticket:
  - `Code review: awaiting reviewer`, with no state word;
  - `Security review: pending` and `Ticket review: pending`;
  - a `### Code review: pending` heading or plain paragraph;
  - `(batch final)`;
  - `REJECT, not yet fixed`, because "fixed" counts as a resolution;
  - an unindented wrapped pending line;
  - a landed ticket with no Review lines.
  Parse against an allowlist of the whole line and fail anything else.
- Wait ratchet. These pass:
  - Rust `elapsed() <= limit`;
  - `use std::thread::sleep as nap`, because aliases resolve only in Python;
  - `Instant::now() < deadline` with a `checked_add` deadline;
  - Python `t = time; t.sleep(1)`.
- Stdio guard. These pass:
  - `.stderr(Stdio::inherit()).output()`, because `.output()` is exempted before the inherit check;
  - `.stdout(std::io::stdout())`;
  - `let s = Stdio::inherit(); … .stdout(s)`.
  The fail-closed branch in `spawn_pattern` is `pass`, so the ticket's "unknown spellings fail" claim is false.
- 1258 lists its report-only gaps in a Review line only. Give it a Deferred gaps section, and name the new-step and `BASH_ENV` holes there.

## Records and licensing

- The licensing pass re-dated 44 sources. 34 changed only the date. The record says the evidence lives in agent session archives (`2026-09-27-source-licensing-review-pass.md:72-74`), which are not in the repo. Commit a per-source table with URL, access date and finding. A date moved without cited evidence does not count as a review.
- The licensing counts say 44 unchanged, 2 changed and 4 unverifiable. That totals 50, and 48 sources were due. The correct split is 42, 2 and 4.
- The retry trigger for the four unverifiable sources has no owner. It gives the warning date as 2026-12-15. The guard warns after 300 days, which is 2027-01-15 (`tests/test_source_licensing_docs_contract.py:213`). Those four still fail on 2027-03-21. The claim that the 2027-03 crossing is handled is false.
- PharmGKB is marked unverifiable. CPIC was verified on the same ClinPGx policy page. Reconcile the two calls.
- `docs/reference/sources.json` repeats a sentence in the mychem-info and mydisease-info entries. The myvariant-info entry keeps an old COSMIC sentence that contradicts the new one.
- `2026-09-26-review-follow-ups-for-1242-through-1254.md` still lists items as open until 1257 or 1258 lands. Both have landed. Its fixed lines cite tickets, not commits.
- The 2026-09-27 dispositions cite a wrong commit for 1255, which merged at `9148e874`. Gate `5754b910` is not on main.
- The Mac run's aspirin check resolved to a five-ingredient combination product, so it never tested the single-ingredient anchor path. File it as its own issue. The DDInter file still says "Ian runs these" (line 6), and its Resolved heading is split across lines 44-45.
- Ticket 1260 lacks Retained behavior, Proof and Deferred gaps sections, and names no code paths.
- The GenCC wrong-owner test uses an unreadable directory as a stand-in (`src/sources/gencc/store.rs:1119`). Say so in the disposition.
- `2026-09-24-yellow-build-overlaps-another-repository-gate.md:63` has a stray fragment.
- The dispositions are hard-wrapped and use shorthand such as "b2", "b3" and "the 1250 fix merge".

## Main's run history

CI is green at `d5bcdd69`. Nine of the ten CI runs before it failed: the stdio flake three times, the wait ratchet twice on the flake fixes, a mistyped checkout pin three times, and a clippy error three times. The "main is green" report rests on one run.

## Disposition (2026-09-29, tickets 1265-1270)

The changelog gate's dated-record bug: fixed (ticket 1265, merges
b1393806 via 535747d9's chain; gates 21d42b2a, b1393806; the
regression test drives record_tickets end to end). #287 confirmed:
fixed (the wheel job builds --release and the smoke exercises the
asset paths positively, ticket 1266 at 80095256; #286 and #287 are
in 1253's close list). Job order: fixed (PyPI last; the tap waits
for the public release; ticket 1266). The stdio test: fixed at the
mechanism (stdin held open until replies arrive, ticket 1268 at
f86a548e; the flake issue carries the rmcp citation). The three
JSON note gaps: fixed (ticket 1268; GWAS excluded by its NoStore,
now pinned structurally since reqwest hides request extensions).
The spawned-fetch note loss: fixed (ticket 1268). The TLS retry
and probe counter: fixed and decided respectively (ticket 1268).
The checks: closed (ticket 1269 at 346068be and 535747d9's chain;
every named bypass, plus the raise-review mechanism Ian's
direction demanded). The licensing evidence: committed in the repo
(docs/reference/source-licensing-evidence-2026-09-27.md, corrected
in c1b2ab0d; CIViC and WikiPathways verified, PharmGKB reconciled,
one unverifiable source remains). The counts: corrected (45
verified, 2 changed, 1 unverifiable after the corrections). The
records: citations now name commits; the open-until lines closed;
the 1255 citation corrected to 9148e874; the aspirin gap filed;
1260's sections concrete; the dispositions unwrapped. The dry-run
mode: DECIDED against, per Ian's 2026-09-28 message — PyPI-last
makes a failed tag run deletable and retryable, so a separate
dry-run lane adds cost without adding safety; the manual dispatch
keeps its container-only path.
