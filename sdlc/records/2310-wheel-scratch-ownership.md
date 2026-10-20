# 2310 — Installed Linux wheel scratch cleanup

Keep the existing wheel build, installation and product smoke commands. Fix only ownership-safe cleanup of the verified temporary run directory, including when a stage fails. Preserve the original stage exception if cleanup also fails.

The unchanged recipe in scripts/check-linux-wheel.sh completes both check=True subprocess stages in Gator 2308/1792530689487-1311653, then fails TemporaryDirectory cleanup on container-created target/CACHEDIR.TAG. Its overall exit remains 2. Continuation 2308/1792531418898-1375131 repairs only its owned scratch and passes; the wheel was already removed during partial cleanup, so no retained-wheel rerun is claimed. Full source, command, traceback, stage output and cleanup receipts remain under existing experiment 2102-biomcp-2258-final-live-replay/security-pins/2308.

Before production edits, candidate 8d09f68f2 adds a bounded ownership regression against the unchanged script. Its Gator job imports the exact bundle on published a5314c569 and performs tiny container-owned file operations using the existing pinned image and repository/ticket labels. No package cold build is needed to reproduce this class. The proposed repair removes only verified scratch contents through that image in finally, then lets the host remove its empty directory. No new privilege or general ownership change is authorized. Success, intentional stage failure, cleanup failure and an unrelated symlink sentinel must retain their correct outcomes.

The red control, corrected focused checks, routine qualification and fresh review are pending here. This ticket remains a final release blocker; prior dependency tickets may cite their verified build and smoke stages without calling their failed aggregate commands passing.

## PM source-prevention ruling

The PM supersedes the proposed after-the-fact cleanup approach: run the wheel container as the host uid and gid, or restore ownership of its exact target before container exit, so root-owned files do not remain in host scratch. No additional cleanup container is authorized as the preventive fix. This is another environment-leak case under the 2299 root cause record. Keep real ownership and failure-preservation controls; choose the narrow compatible source fix after inspecting the pinned image.

## Ownership controls and attribution

Final direct preimage job 2310/1792531661790-1465829 reproduced PermissionError after successful bounded fixture stages with the real pinned container. Source 84cf211e8 passed Gator 2310/1792531879220-1469287: twelve runner controls include real container ownership after success, build and smoke failures, ownership-restoration failures and combined failures. The unrelated symlink target has a deliberately different owner; its content, mode, uid and gid remain unchanged. No cold wheel build was repeated. Earlier aggregate focused attempt passed nineteen cases but then lacked Ruff; that setup failure stays separate.

The source fix uses an EXIT trap inside the existing build container, restoring only this run target and wheel output with physical chown behavior before container exit. The host finally preserves the primary exception if its ordinary cleanup also fails. No later cleanup container is the preventive implementation.

The lander found the same recipe hardcodes ticket 2288 on containers serving later jobs. Correcting this ownership label is part of this ticket: the documented PM_JOB contract is NNNN SHA; validate and use its ticket token, with release for non-job use. The unstarted 84cf211e8 routine waiter was verified and canceled before any command ran so the next routine can qualify the final label-corrected source. The existing ownership-control proof remains bound to its exact tested bytes.

## Final source and controls

Final candidate c92fcf0a338314dc9697a0bd749a210d84c848ed validates PM_JOB as numeric ticket plus revision and labels both containers with that ticket; non-job use labels release. Gator 2310/1792532164644-1491629 passed all twelve focused runner cases and Bash syntax. The real root-container cases establish ownership restoration before host cleanup and preserve an unrelated differently owned symlink target; label controls cover job and release contexts. The production fix adds no CPU tuning or later cleanup container.

M5 routine 2310/1792532173981-15039 runs on the exact final source. Its retained final-v3 bundle SHA256 is 64f3335ca9f452d83b04a655a03d4388ee0c3650965172dd0c4d8c22eac08af9 and runner SHA256 is 03c3b8eb85d8fe0f39190e50637e412dcba33288033c3f14561a468ece24c85d. Raw controls, job records and bundles are durably retained under wheel-scratch/2310 in the owning admission experiment. Fresh review and the routine result remain pending here.
