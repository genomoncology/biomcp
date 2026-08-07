---
flow: quickfix
priority: 6
---
# Move misplaced biomcp planning files out of biomcp repo and gitignore the path

`repos/biomcp/planning/biomcp/planning/{learnings.md,quality-bar.md}` were accidentally committed inside the biomcp repo (latest by ticket 306; `learnings.md` predates that). The intended location is the team planning folder `~/workspace/planning/biomcp/planning/`, which is gitignored and synced to Obsidian.

Completed under March on 2026-04-26, as March ticket 317. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/317-move-misplaced-biomcp-planning-files-out-of-biomcp-repo-and-gitignore-the-path
