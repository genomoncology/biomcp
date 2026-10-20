# 2310 — Installed Linux wheel scratch cleanup

Keep the existing wheel build, installation and product smoke commands. Fix only ownership-safe cleanup of the verified temporary run directory, including when a stage fails. Preserve the original stage exception if cleanup also fails.

The unchanged recipe in scripts/check-linux-wheel.sh completes both check=True subprocess stages in Gator 2308/1792530689487-1311653, then fails TemporaryDirectory cleanup on container-created target/CACHEDIR.TAG. Its overall exit remains 2. Continuation 2308/1792531418898-1375131 repairs only its owned scratch and passes; the wheel was already removed during partial cleanup, so no retained-wheel rerun is claimed. Full source, command, traceback, stage output and cleanup receipts remain under existing experiment 2102-biomcp-2258-final-live-replay/security-pins/2308.

Before production edits, candidate 8d09f68f2 adds a bounded ownership regression against the unchanged script. Its Gator job imports the exact bundle on published a5314c569 and performs tiny container-owned file operations using the existing pinned image and repository/ticket labels. No package cold build is needed to reproduce this class. The proposed repair removes only verified scratch contents through that image in finally, then lets the host remove its empty directory. No new privilege or general ownership change is authorized. Success, intentional stage failure, cleanup failure and an unrelated symlink sentinel must retain their correct outcomes.

The red control, corrected focused checks, routine qualification and fresh review are pending here. This ticket remains a final release blocker; prior dependency tickets may cite their verified build and smoke stages without calling their failed aggregate commands passing.
