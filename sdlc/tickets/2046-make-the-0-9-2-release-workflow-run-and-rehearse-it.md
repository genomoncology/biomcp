# 2046 — Make the 0.9.2 release workflow run and rehearse it

Status: OPEN.

Milestone: 0.9.2

## Outcome

The release workflow that the 0.9.2 tag will run loads on GitHub, every smoke leg checks the version it claims to check, the changelog says only what landed, and one rehearsal in the public scratch repo runs the exact candidate workflow end to end before the real tag.

## Evidence

Filed 2026-10-10 from the independent review of `origin/tickets/landcheck-C` at `ac4c6739a` over main `c6d972d47`. Every item here holds the tag under Ian's clean-release ruling.

- `.github/workflows/release.yml` has two `with:` keys on the build checkout (lines 105 to 111) and the pypi-build checkout (lines 225 to 231). GitHub refuses the file: Release runs 38049457592 (landcheck-C) and 38041251226 (tickets/2038-rc-release-smoke-versions) failed in zero seconds with "workflow file issue" and no jobs. PyYAML keeps the last key, so `tests/test_release_workflow_provenance.py` stays green and `fetch-depth: 0` silently disappears.
- The manylinux wheel smoke (lines 375 to 383) puts `printf '%s'` and `tr -d '\r\n'` inside the single-quoted `bash -c` script, so the quotes close early, and `docker run` passes no `-e TAG`. The container receives `tr -d rn` and an unset `${TAG#v}` under `set -u`. Running the captured script without TAG prints `TAG: unbound variable` and exits 1, so every Linux wheel-smoke leg fails and nothing publishes. Head `6116e2bbb` spliced the tag in on the host and worked; the fix broke the splice.
- The canonical lint gate already caught both. CI run 38041252124 at `9a1f5ec83` and run 38049458224 at `ac4c6739a` failed canonical-gates with actionlint lines `release.yml:110:9: key "with" is duplicated`, the same at line 230, and shellcheck SC1078 and SC1012 at line 374. The 2038 version-bump section on landcheck-C says "CI green at the fixed head". The last green run, 38014529319, was at `6116e2bbb`.
- `fetch-depth: 0` is not needed: actions/checkout with `ref: v0.9.2` fetches that tag even at depth one, and `git describe --tags --exact-match` then prints `v0.9.2`. The block added to container-publish (lines 670 to 675) serves nothing, since that job builds no binary.
- The current release workflow has never run end to end. The last rehearsal in `genomoncology/biomcp-release-rehearsal` was run 36896962624 on 2026-10-01 against v0.9.1, before ticket 2024's runner pins and the with-build-identity routing (`c037bddb3`, `d07642eb3`). Ticket 2038's Outcome asks for a passing rehearsal on the candidate.
- The changelog misses landed work. 2042 (past-the-end positions, UniProt unreachable, more than 32 SnpEff rows), the 2038 code lane (terminal-only WHO path, kept filters, MyGene and UniProt as new sources), and 2044 (MCP errors carry no local path, the refused hint keeps every flag) have no 0.9.2 bullet. `scripts/check-changelog-coverage.py` counts only `Land NNNN:` and `Merge …tickets/NNNN-` subjects, so "Land tickets/2042-fix" and "Land 2044 and 2045:" escape it. `tests/test_docs_changelog_refresh.py` stops at 2037.
- Bullets that say more than landed:
  - `CHANGELOG.md:27` (2031) says `get drug NAME` returns the named drug or an honest no-match; open 2039 records "Pain Relief" fusing acetaminophen with TRPV1 targets.
  - The 2043 bullet near line 32 says a refusal runs the discover rescue once instead of twice; the cache never saves a call on the refusal path (ticket 2047).
  - Line 51 (2035) says the fourth review closed or answered every remaining finding; 2038 finding 7 lists the ones it did not.
  - Line 49 (2030) says the slow-timeout kill comes at 120 seconds; `.config/nextest.toml` sets `period = "120s", terminate-after = 2`, about 240 seconds.
- `architecture/technical/overview.md:271` still names 0.9.0 and 0.9.1-dev.1, and the comment at `release.yml:553` still says 0.9.1.

- Starts from: `origin/tickets/landcheck-C` at `ac4c6739a`.
- Keeps: the exact `biomcp X` comparison on every smoke leg, host-side line-ending stripping, the version files at 0.9.2.
- Changes: fold any checkout option into the existing `with:` or drop the added blocks; pass the tag into the container with `-e TAG` and double quotes inside the script; add a duplicate-key check across the whole workflow to the provenance tests; add a check that every variable read inside a `docker run … -c` script is passed in; add the missing bullets and correct the four above; count every landing subject shape in the coverage gate; land only on a green CI run of the exact landing tree; run one rehearsal of the candidate workflow in `genomoncology/biomcp-release-rehearsal` and record its run ID.
- Proof: a test that fails on `ac4c6739a` for the duplicate keys; the captured container script exits 0 with the tag passed; canonical-gates green on the landing tree; a green rehearsal run ID on the candidate SHA.
- Defers: nothing.

## Build record (2026-10-10)

Built on `tickets/2046-work`: `origin/tickets/landcheck-C` (`30e233c2e`, whose tip already merged the duplicated `with:` keys) merged with main (`239b16f5f`, which carries the 2048 contract-test fix), then the Changes above. The three added `fetch-depth` blocks are gone (version-check keeps its own, which the changelog scan needs), the manylinux smoke passes the tag with `-e TAG` and uses only double quotes inside its script, and the one changed step hash was repinned through the provenance module's own `_step_sha`.

Red probes, both run through `yr` on the build host and deleted after:

- `probe/2046-red-a` at `be6cf1024` (`ac4c6739a` plus the new tests): the whole-workflow duplicate-key check failed on the real duplicated `with:` keys, and the docker-run variable check failed on the smoke reading `${TAG#v}` with no `-e TAG`, through the broken quotes. This is the Proof line's failing-on-`ac4c6739a` test.
- `probe/2046-red-b` at `595e33cba` (the merged tree plus the new tests, fixes withheld): the two new landing-shape coverage tests failed (the `Land tickets/2042-fix` and `Land 2044 and 2045:` subjects escaped the gate), the changelog-refresh ticket set failed against the pre-fix changelog, and the provenance suite failed on the `pypi-build` checkout hash, which the `fetch-depth` block had silently drifted.

The captured container script's tag mechanics hold, verified through `yr` on the build host against the pushed branch: with `TAG=v0.9.2` in the environment the trimmed-line comparison exits 0 on a `biomcp 0.9.2` line, and without a tag value the script exits 1 instead of comparing garbage (with TAG absent entirely it dies on the unbound variable the Evidence line records).

Verification: CI caught two real test bugs on the way green, each fixed with its own commit — run 38057186451 at `7e10466b8` failed the planted-duplicate proof (its quiet-parse assertion checked a string against dict keys), and run 38059743925 at `745779e9e` failed `test_technical_and_ux_docs_match_current_cli_and_workflow_contracts`, which still pinned `v0.9.0 is the latest published release` in the Release Pipeline section; the pin now states the corrected facts. At head `06b9f24fa`, under the checkout lock with the head verified stable before and after: `make lint` green, and the full offline pytest lane including `needs_binary` green at 1252 passed, 3 skipped. The Rust nextest stage hangs on the three `cli::update` archive tests on the shared build host (240-second slow-timeout kill, reproduced serially), the flake ticket 2044's verification records for this box; CI ran that stage green on this branch's heads.

## Candidate

The final head of `tickets/2046-work` — the commit that carries this record — is the 0.9.2 tag candidate. The rehearsal in `genomoncology/biomcp-release-rehearsal` is the coordinator's step after this branch lands on main, per the task split that filed this lane.
