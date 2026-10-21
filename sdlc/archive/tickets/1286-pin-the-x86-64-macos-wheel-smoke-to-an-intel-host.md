# 1286 — pin the x86_64 macOS wheel smoke to an Intel host

Filed 2026-10-01 from the second release rehearsal (scratch run 36885506628).

## The finding

The rehearsal's wheel-smoke (macos-latest, wheel-x86_64-apple-darwin) leg failed at install: "biomcp_cli-0.9.1-py3-none-macosx_10_12_x86_64.whl is not a supported wheel on this platform." The job's runner image log shows `Image: macos-26-arm64` — on BOTH attempts of the first run (14:30 UTC) and again in the second run (16:08 UTC), so the label served arm64 from the first rehearsal onward and no mid-day switch was ever observed. The earlier claim that this leg "installed fine on an x86_64 host" in the first run was inference from another leg's log, not an observation of this leg's log (corrected 2026-10-01 on the sixth review's reading; the leg never reached the DDInter line). With the DDInter fix (ticket 1285), the other four legs pass in the second run; only this leg kept failing, which the arch mismatch explains. The build and pypi-build legs are unaffected in output correctness: they cross-compile with an explicit cargo `--target`, and the refused install proves the wheel carries genuine x86_64 tags. Only the smoke assumes the host matches the wheel.

## The fix

The x86_64-apple-darwin wheel-smoke leg runs on `macos-15-intel` (the documented Intel image; macos-latest and macos-15 are arm64 now, and pinning macos-15 alone does not select Intel). The matrix pin in the provenance test is updated in the same commit. When Intel runners retire, the leg moves to Rosetta on an arm64 image; that migration is future work, not part of this fix.

## Evidence

- Run 36885506628: the failing leg's log carries "Image: macos-26-arm64" and the pip refusal above; the four other wheel-smoke legs passed with ticket 1285's sync line in.
- Runner tables: GitHub's runner reference lists macos-15-intel (and macos-26-intel) as Intel, macos-latest and macos-15 as arm64.

## Review

- Code review: ACCEPT 2026-10-01 dispatch 944e8046-1a7c-48ea-9719-6295869163a3 (Claude vendor, read-only) — every macos-latest use in the repo checked (two cross-compile legs and one test pin, none run the binary), the arm64 leg confirmed correct as-is, the matrix pin verified against the workflow, no other hardcoded runner labels in docs. One non-defect note: the changelog bucket is a judgment call.
