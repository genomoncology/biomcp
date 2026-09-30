# Review of the third 0.9.1 go request

Filed 2026-09-30 from an independent read-only review of main at `2523f29b`, range `d9ae247d..2523f29b`. Two fresh reviewers covered release readiness and re-ran mutations for every fix claimed. Reviewers used scratch clones with scratch tags, offline tests and read-only GitHub queries.

## Verdict

Do not tag `2523f29b`. The GHCR `latest` tag still would not move, and the changelog leaves out a feature that ships for the first time in 0.9.1.

## What holds

- `2523f29b` is a merge commit. Main run 36761184223 and branch run 36758619935 at `ea490997` passed all eight jobs.
- A scratch tag passes version sync, release versions and the changelog gate, which counts 55 tickets.
- `scripts/should-move-latest.sh` orders versions correctly when it runs: a draft v0.9.1 moves past v0.9.0, a published v0.10.0 blocks it, `v0.9.2-rc.1` stays, and v0.9.10 beats v0.9.9.
- Job order holds: pypi-publish needs container-publish (`release.yml:480`), the Docker steps run before `gh release edit --draft=false` (line 730), and homebrew-tap needs publish-release (line 507).
- The classifier tests catch each of `docs/`, `docs/charts/` and `skills/` being re-admitted. The evidence guard catches a collapsed row. The wait ratchet runs in repository-contracts (`ci.yml:333`).
- The #282 and #250 changelog entries read correctly, and schemas sit under fixes.
- Records 1279 and 1280 exist with real green runs. Records 1265 and 1268 are corrected. Ticket 1257 is clean. `.review-artifacts/` is ignored.
- The review dispatch `598c0b9a` is genuine and read-only.
- The main CI failure at `9bfb4f79` came from the reviewer's own issue file, which named branches that the zero-coupling check forbids. `2523f29b` fixed it.

## Release blockers

1. The `publish-release` job has no `actions/checkout` step (`release.yml:684-731`), and line 720 runs `scripts/should-move-latest.sh`. On the runner the file does not exist. The call fails, the `if` takes the else branch, the step logs "latest stays put" and exits 0. `latest` stays on 0.9.0. `tests/test_release_workflow_provenance.py:168` only checks that the script name appears. Add the checkout and a test that fails when the job lacks one.
2. Records 1202, 1205, 1213 and 1214 carry `backfill: v0.9.0`, but their work is not in v0.9.0. v0.9.0 is `a4503038`, dated 2026-09-17. The cell-line commits (`37336c22`, `eed4f2c1`, `fd6a100c`, `bccd2871`) date from 2026-09-18 on and are not ancestors of v0.9.0. `src/cli/cell_line/` is missing from v0.9.0. At `2523f29b` the command exists (`src/cli/commands.rs:46`). So 0.9.1 is the first release to ship the cell-line entity and its PharmacoDB, HPA and ChEMBL sections, and the changelog never mentions them. Ticket 1281 lines 3 and 7 say the code landed long ago, which hides that it was never released. Remove the markers and add a changelog entry for the new entity.

## Gate and test holes

- `record_is_backfill` (`scripts/check-changelog-coverage.py:55-66`) accepts any non-blank value. A record for a new ticket with the marker and no merge commit passes. Check that the named tag contains the record's commits, or remove the marker.
- `should-move-latest.sh` fails open. `mapfile < <(gh … | grep … || true)` hides a `gh` failure, and an empty list moves `latest`. Fail the step when `gh` fails. No test compares a two-digit part, so a string comparison passes all five tests. Add v0.10.0 against v0.9.0.
- The raise-review gate still accepts "ACCEPT - will fix", "ACCEPT (pending)", "ACCEPT; will fix later" and "ACCEPT, to be confirmed" (`tools/check-test-wait-ratchet.py:309-311`). A list of banned words cannot close this. Accept only `ACCEPT` followed by nothing, a date, or a dispatch reference, and test the four phrases.
- The evidence guard's comment claims a registry check that the code never runs (`tests/test_source_licensing_docs_contract.py:494-506`). A row renamed to a source that does not exist passes. Compare row names to `sources.json`.

## Records

- Record 1281 does not exist.
- Ticket 1276 has no reviewer dispatch. Its line 17 defers to ticket 1281's history, and 1281 never names 1276. Record 1276 keeps a prose ACCEPT. Ticket 1280 line 13 still says 1276 carries a real verdict.
- Ticket 1281 overstates the ChEMBL carry. The deleted branch cited the ChEMBL about page as the terms URL. Main names "the about page" with no URL (`docs/reference/source-licensing.md:243` and `sources.json`). The LICENSE URL and licence text did carry.
- `sources.json` was re-indented as a whole in this range, a 3,138-line diff for two changed fields. Keep formatting changes out of content commits.
- Ticket 1269 and record 1264 (from line 7) are still hard-wrapped. `tests/test_ci_classify_push_script.py:3` says "P1".
- The CHANGELOG line 36 ending "the flat MCP tool schemas are published for client authors" repeats line 7.

## Ian's direction, 2026-09-30

- Ship the cell-line entity in 0.9.1 and document it in the changelog as a new entity type.
- BioMCP may search and report public research data, including cell-line data. Nothing in BioMCP needs to be cancer-only.
- BioMCP does not process primary data files locally, such as FASTQ files.
