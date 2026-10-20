---
priority: 2
---

# 2288 — Run the installed-wheel smoke on a fresh home in the routine check

Status: OPEN.

Milestone: 0.9.4

## Outcome

The routine full main check builds the Linux wheel, installs it in the same manylinux container release.yml uses, and runs the release.yml installed-wheel smoke commands under a fresh HOME with no cache ancestors. It uses the same `BIOMCP_DISABLE_SOURCES` profile as release.yml. A fresh-root, packaging or installed-command failure fails the routine check, so the GitHub release rehearsal is never the first place one appears.

- Proof: Breaking fresh-root directory creation on a branch fails the routine check. The routine check and the release rehearsal run the same smoke command list.

## Evidence

In 0.9.4 the first full rehearsal found three problems no local check had run: installed Linux wheels failed `cache stats --json` on a fresh root (2286), the DDInter smoke could not pass under the commercial-restricted profile, and the article smoke called Semantic Scholar through enrichment. Each cost a rehearsal and a new qualification round.

Reuse the release.yml smoke command list from one shared source, so the local and release smokes cannot drift. Run serially with the existing caps and pacing. Use no keys. Keep KEGG and Semantic Scholar uncalled. Where a public provider call is needed, keep it to the commands release.yml already runs.
