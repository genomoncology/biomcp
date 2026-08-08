# The March harvester mispads ids and lets archive ids collide

Severity: blocking — the factory quarantines this whole repo at
every sync ("0043: duplicate id in input", "0073: duplicate id in
input"), so registration is complete but no ticket here can ever
import until this is fixed. Nothing else is affected; quarantine
isolates the repo by design.

Two distinct defects, found 2026-08-08 at first registration:

1. **Unpadded source ids are misfiled.** `tickets.json` holds four
   ids stored without padding: `48`, `49`, `59`, `72`. March ticket
   `59` ("Repo cleanup — gitignore, rename, consolidate") was
   written to `archive/0073-repo-cleanup-gitignore-rename-consolidate.md`
   — the file's own body correctly says "March ticket 59", but the
   filename says 0073, colliding with the real 073 (BioASQ gaps).
   Check where 48, 49, and 72 landed too; a misfiling that does not
   collide is still a misfiling, just a silent one.

2. **March itself reused id `043`** — two completed tickets share
   it (opentargets-drug-indications, 15:43, and EMA command
   surface, 19:28, both 2026-03-24). Queue identity is channel+id,
   so one must be renumbered. Suggested policy, for this and any
   future collision: keep the earlier completion on the original
   number, move the later one to original+800 (here: EMA becomes
   0843) — far above March's 690, lineage visible, and note the
   renumbering in the file body.

Fix in the conversion script, not by hand — it runs again for
other teams — and add both cases to its coverage report.

Done when: the factory's sync imports this repo without a
quarantine line, visible in `~/.local/share/sdlc/cron.log` on
beelink (or ask the sdlc side to confirm).
