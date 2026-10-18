---
priority: 2
---

# 2198 — Keep release notes about BioMCP and credit reporters

Status: OPEN.

Milestone: 0.9.3

## Outcome

The 0.9.3 release notes describe user-visible changes to BioMCP, credit every GitHub reporter whose issue 0.9.3 answers, and the public repo names no private tooling.

- Evidence: release review of 7025e117: the 0.9.3 changelog section, which the release publishes as GitHub release notes, carries about 65 internal CI and process bullets and refers to private job tooling and gate hosts; `AGENTS.md` lines 51 and 106 and `tools/measure-0-9-3-baseline` name the private tooling. #291 (Seungwon Lee) is credited; #290 (Miika Ahdesmaki), which asked for source disabling, and #292 (rizakardas), the Windows lock error, are not. The cache docs say broadened permissions are repaired at start-up; after `.tree-secured-v1` exists they are not (the root stays 0700).
- Changes: move internal bullets out of the published section (or into a short internal-changes line without tool names), remove private tooling names from public files, add the two credits in the existing style, and make the permission docs state what BioMCP actually repairs.
- Proof: the release-notes extraction for 0.9.3 shows only user-facing bullets with three credits; a repo search finds no private tooling names in public files.

## Progress

- 2026-10-18 started
- 2026-10-18 next: Outcome: publish user-visible notes with all three reporter credits, remove private tooling prose and correct permissions docs. Baseline extraction exposes internal bullets and credits only291; targeted public-file search confirms private tooling names. Change changelog, guide, permission docs and prose-only comments. Initial job1792322126630-59106 passed153focused tests and mechanical checks; final wording check queued.
