---
flow: build
priority: 8
---
# Guide the exact-variant literature workflow and detect stale installed BioMCP skills

The intended exact-variant workflow is hard for agents to discover, and installed guidance can silently drift from the running binary. Current `main` does not suggest `variant articles` after a recognizable `MSH2 p.L341P` article query, and the installed Codex skill still teaches retired `biomcp article search/get` grammar plus a nonexistent article `-v/--variant` flag. `biomcp skill render` and the installed `SKILL.md` have different hashes, but BioMCP records no installed version/hash and has no read-only status command to explain the mismatch.

Completed under March on 2026-07-21, as March ticket 603. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/603-guide-the-exact-variant-literature-workflow-and-detect-stale-installed-biomcp-skills
