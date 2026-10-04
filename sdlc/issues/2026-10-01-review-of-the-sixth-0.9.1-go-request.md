# Review of the sixth 0.9.1 go request

Filed 2026-10-01 from an independent read-only review of main at `d7176bce`, range `9e960ee3..d7176bce`, and of the release rehearsal in the scratch repository. Two fresh reviewers covered the rehearsal run and this round's fixes and records. They read every attempt's job logs, compared the rehearsal copy with main, re-ran mutations in scratch clones, ran the gates on a scratch tag, and fetched the source terms pages.

## Verdict

The release path is proven for `d7176bce`. The artifacts would be right. Two small things should change before the tag: the changelog date and the false records about the macOS runner. Neither touches the release workflow or scripts, so the rehearsal still holds for the next commit as long as `.github/`, `scripts/`, `Dockerfile`, `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, `src/` and `tests/` are unchanged.

## What holds

- Rehearsal run 36896962624 ran on `9cbadbf3`, which is `de7ceb2d` plus one rehearsal commit. That commit removes the CI, contracts and docs workflows, stubs docs-live, replaces the PyPI upload with `twine check`, and points four image names at the scratch repository. `scripts/` is untouched. `environment: pypi` and `id-token: write` are kept.
- `git diff de7ceb2d d7176bce` is empty for `.github/`, `scripts/`, `Dockerfile`, `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, `src/` and `tests/`.
- Attempts 1 and 2 stopped at container-publish because GitHub refused to start the job for a billing reason. No step ran, so no flaky step passed on retry. Attempt 3 re-ran the failed jobs, and container-publish, pypi-publish, publish-release and homebrew-tap all passed. The upstream jobs passed on their only run.
- The attempt 3 logs show each path ran: the draft download with both checksums OK, both image smokes as uid 65532 printing `biomcp 0.9.1`, the `latest` move, `gh release edit --draft=false`, the Homebrew skip, and `twine check` passing all five wheels.
- The rehearsal caught two real blockers. Ticket 1285 added the DDInter sync to wheel-smoke, and ticket 1286 pinned the Intel macOS runner. Each landed through a merge with a green branch run.
- Main run 36924091357 at `d7176bce` passed all eight jobs, and documentation run 36924091356 passed. biomcp.org serves `d7176bce`.
- A scratch tag passes version sync, release versions and the changelog gate, which covers 64 tickets.
- container-publish has `contents: write`, and reverting it fails a test. Widening the exit-code case to `3|*)` fails a test. The gh guard catches `if ! gh`, `$(gh …)`, `| gh` and `&& gh`.
- Record 1283 exists with real run IDs. The fourth and fifth review files have dispositions for every finding.
- The offline suite passes 824 tests. The zero-coupling check passes.
- genomoncology/biomcp has no v0.9.1 tag or draft. Its latest release is v0.9.0.

## Before the tag

1. `CHANGELOG.md:3` reads `## 0.9.1 — 2026-09-30`. Set it to the tag date.
2. Tickets 1285 and 1286 and record 1286 tell a false story about the macOS runner. Ticket 1286 line 9 says the x86_64 macOS leg installed fine in the first rehearsal and GitHub moved `macos-latest` to arm64 mid-day. In run 36872650687, attempt 1 (job 110416405617) and attempt 2 (job 110418428951) both ran on "Image: macos-26-arm64" and failed at pip install with "not a supported wheel on this platform". Ticket 1285 line 11 says all five legs failed at the DDInter line, which is false for that leg. Correct the records from the logs.

## Not exercised by the rehearsal

- The real docs-live job. The stub removes the checkout, the tag lookup, the retry loop and `scripts/check-docs-live-revision.py`. A unit test covers the script, and biomcp.org serves the tag commit.
- The trusted-publishing upload to PyPI. It is the same command that published v0.9.0.
- The Homebrew tap push, because the scratch repository has no token.
- The `latest` decision against real history. The scratch repository had no earlier releases, so the script saw an empty list. The script's tests cover the real cases.

The checklist's section 4 verification covers PyPI, Homebrew and the `latest` tag after the real run.

## Records and checklist

- Commits `06ba305f`, `b18e1824` and `49f5e61b` landed after the 1284 verdict with no review. `49f5e61b` ticks the rehearsal item and rewrites the fifth review's rehearsal disposition. The reviewer returned "OK with notes", which the ticket records as ACCEPT. Ticket 1284 line 26 says commands and outputs are in the review artifacts, but the reviewer ran no commands.
- Record 1284 still says the scratch copy is private, uses TestPyPI, and has not pushed its tag. Ticket 1284 line 13 says the same.
- `sdlc/release-checklist.md:21` describes a private copy that uploads to TestPyPI. Describe the rehearsal as run: a public scratch copy, `twine check` in place of the upload, docs-live stubbed, the other workflows removed. Say that the real docs-live, the upload and the tap push are checked in section 4.
- `sdlc/release-checklist.md:52` still names ticket 1283 as the worked copy.
- The worked checklist in ticket 1284 says 61 tickets. The gate reports 64. The main CI, live docs and `workspace repos --dirty` items defer their evidence to the go request with no run ID or output.
- Lesser note (a) and the `sources.json` move mixing are marked accepted without Ian's recorded approval.
- "P2" appears as shorthand in a new file.

## Gates and sources

- The raise-review grammar still passes capitalized promises: "ACCEPT by Will Fix", "ACCEPT by Pending Fixes", "ACCEPT by TBD", "ACCEPT by Not Reviewed", "ACCEPT by Sol-will-fix-later" and "ACCEPT dispatch deadbeef-----". "ACCEPT" followed by a newline and "will fix" also passes, because the check reads one line. Leave this for the structured verdict field in 1.0. Ticket 1269 line 18 still describes the older grammar.
- `docs/reference/source-licensing-evidence-2026-09-27.md` lines 55, 56 and 58 say "fifth go-request follow-up" and "fifth go-request round" in a public page. Line 58 says 50 sources and line 60 says 48.
- The Cell Model Passports `terms_url` is the Sanger DepMap data usage policy, which names Cell Model Passports only in its footer. Say why that policy covers it. The DepMap summary leaves out the staff note that the data are not meant for clinical use.
- wheel-smoke now runs `ddinter sync` on all five legs. It downloads about 13 MB of CC BY-NC-SA 4.0 data from ddinter.scbdd.com and keeps it on the runner. Record the judgment that a release test of an open tool fits the non-commercial terms. The release now also stops if DDInter is down. That happens before PyPI, so a retry is cheap.
- `macos-15-intel` is GitHub's last x86_64 macOS image, available until August 2027. Record the date in ticket 1286.

## Housekeeping

- The jobs on `ubuntu-latest` move to Ubuntu 26 from 2026-10-19. Tag before then, or pin the runner.
- `ghcr.io/genomoncology/biomcp-release-rehearsal:0.9.1` and `:latest` are public and pullable without login. They hold a real `biomcp 0.9.1` binary built from a scratch copy. The repository description says the rehearsal is deleted after each run. Deleting the package needs Ian's authorization.
- The scratch repository was made public without Ian's approval. Attempts 1 and 2 were blocked by a GitHub billing stop on the organization account.

## Diff check of the tag candidate, 2026-10-01

`992df4c8` is a merge commit. Main run 36946458067 passed all eight jobs, and the live docs serve it. `git diff d7176bce 992df4c8` changes the changelog date, the matching `CITATION.cff` date and its three test literals, and the corrected records for tickets 1285 and 1286. Nothing under `.github/`, `scripts/`, `src/`, `Dockerfile`, `Cargo.toml`, `Cargo.lock` or `pyproject.toml` changed, so the rehearsal still holds. No v0.9.1 tag or draft exists. The reviewer returns GO for tagging `992df4c8`.

## Ian's direction, 2026-10-01

- Tag v0.9.1 on `992df4c8`. The changelog date stays as it is.
- Do not run release rehearsals in a private repository, where runs cost money. A rehearsal runs only in a public scratch repository, where runs are free.
- The remaining records, checklist and source items in this file go to a sweep after the release.
