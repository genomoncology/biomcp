# Windows manual skill repair

Ticket: [2358](../tickets/2358-restore-windows-atomic-skill-repair.md)

Accepted product e6abc1752e3889279d7e8c70b590b035beb2a08c retains the Windows forced-repair refusal. Its message, help and documentation explain how to back up unrelated files outside the managed folder, remove that folder, install again and restore the saved files.

M5 job j4577 reproduced the missing Windows instructions in actual install help. Job j4578 passed 19 affected Rust cases, six documentation checks, formatting, Ruff, Clippy with warnings denied and feature links. It executed base 1d29e9f8bb581119f6df1620db8ef36fac54aab3 plus retained patch SHA256 2207c447518ea701feb038d2facb83a30e28994daa936ce8317d6cc64cd13460. The pretransport patch, tested source diff and committed diff to the accepted product match exactly. Yellow ticket-evidence/2358/development-manual-repair retains both original drivers, scheduler receipts, complete logs, patches and source binding. All ten files were verified against the local originals before cleanup.

Native Windows [run 39837405107](https://github.com/genomoncology/biomcp/actions/runs/39837405107) passed at actual source 81369f468d99af780302cf16c70791845335a7cc. Its product modes and blobs match the accepted product; only the review marker differs. The raw log confirms that the forced-refusal preservation and manual-reinstall case, install-help case and fresh-install case all passed. The Windows job passed 15 candidate cases and one released-version comparison. The classifier also passed; eleven other jobs skipped. Yellow ticket-evidence/2358/native-39837405107 retains the complete raw proof and source comparison. Its verified manifest SHA256 is 257fbc5e23ad29c199f1209d7cb8636df60e8f3466c8d05d4b7d0e7d653b3f88.

This run used the default development test profile and produced no artifacts. It supplies no optimized installed-wheel proof or complete release qualification. Ian's landing hold remains in force; this evidence authorizes no merge or release.
