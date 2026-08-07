---
base: c44f7cfbec0db2042bcd515f527a868370beaa51
head: 116fa11b3372690a566b5154fb2a833e84aa4614
---
The intended exact-variant workflow is hard for agents to discover, and installed guidance can silently drift from the running binary. Current `main` does not suggest `variant articles` after a recognizable `MSH2 p.L341P` article query, and the installed Codex skill still teaches retired `biomcp article search/get` grammar plus a nonexistent article `-v/--variant` flag. `biomcp skill render` and the installed `SKILL.md` have different hashes, but BioMCP records no installed version/hash and has no read-only status command to explain the mismatch.

Imported from March ticket 603. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/603-guide-the-exact-variant-literature-workflow-and-detect-stale-installed-biomcp-skills
