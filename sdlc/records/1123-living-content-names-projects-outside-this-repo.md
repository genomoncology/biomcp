---
flow: build
priority: 2
---

# Current prose names private projects outside BioMCP

## Outcome

BioMCP's current experiment conclusions and open issues explain their evidence
and downstream consumers without requiring knowledge of private projects or an
unopenable path in another repository.

## Current facts

At `27dd2090` on 2026-09-05, a case-insensitive tracked-file scan found five
stale current-prose passages, not the four previously claimed:

- `architecture/experiments/structural-variant-article-annotations/explore.md`
  names two private projects as a future schema dependency.
- `architecture/experiments/structural-variant-article-annotations/harden.md`
  names the same projects as consumers and cites a missing cross-repository
  spike-plan path.
- The 2026-08-26 drug-mechanism and search-all-pathways issues both cite the
  same external marketing capture by its workspace-repository path.
- `sdlc/issues/2026-08-27-article-downloads-to-user-directory.md` names a
  separate private paper-library tool as a downstream consumer. Withdrawn
  ticket 1081 recognized that private identity in a different active-ticket
  occurrence, but this open-issue occurrence was omitted when 1123 was split
  from it.

The two capture issues already contain the observations, reproducer, version,
date, and code/data analysis needed to stand without the private file. The
article-download issue likewise explains the other tool's relevant capability.

This is not a ban on all external names or paths. The scan also found truthful
BioMCP hosting and release integration, biomedical providers and dependencies,
historical experiment execution paths, internal onboarding instructions, test
fixtures, and a reproduced workstation path in an open path-disclosure issue.
Those references either identify a public dependency, preserve evidence, or
demonstrate the defect itself. Cargo excludes `architecture/` and `sdlc/`, and
the package-shipped `README.md`, `AGENTS.md`, `docs/`, and `skills/` surface has
none of the five stale private references.

## Scope

- Replace the structural-variant project names with a role such as
  "a downstream alteration-grammar consumer". In `harden.md`, describe the
  unavailable plan without retaining its external path.
- Restate both capture origins as an external review capture while preserving
  the BioMCP version, capture date, observation, and subsequent analysis.
- Describe the article-download consumer by capability (a separate
  paper-library tool), preserving why BioMCP's proposed output closes the gap.
- Add a static regression contract to the existing
  `tests/test_documentation_consistency_audit_contract.py`. Scan current
  Markdown under `docs/`, `architecture/`, and `sdlc/issues/` for the
  distinctive private markers (`Trials3`, the phrase `rolodex tool`, and the
  external capture path). Do not ban the bare word `Nucleus`, which is valid
  biomedical prose; `Trials3` identifies the private project pair in both
  target passages. Keep the missing-plan assertion scoped to the
  structural-variant hardening passage because other experiment reports
  truthfully record historical prompt inputs.

Do not rewrite `sdlc/records/`, `sdlc/tickets/archive/`, other experiment
execution evidence, planning/onboarding instructions, test fixtures, provider
or dependency names, BioMCP's public `genomoncology` hosting/attribution, or the
path shown as defect evidence in the article-download issue. Do not copy the
external capture into the repository: its relevant evidence is already stated
locally, and the repository has no established right or need to redistribute
the private capture itself.

## Acceptance

1. Add the generic static contract first and show it fails on exactly the five
   passages above, then passes after the prose changes. The check must cover all
   three current-content roots rather than hard-code only the five filenames.
2. The two structural-variant passages retain the need for downstream schema
   review and the reusable-consumer rationale, without the private names or
   missing external plan path.
3. Both 2026-08-26 issues retain the captured behavior, version/date, and local
   technical diagnosis without an external repository path.
4. The article-download issue retains the separate tool's arXiv-only limitation
   and the reason BioMCP fills that gap without naming the tool.
5. Focused pytest, `make lint`, `make test`, and `make spec` pass.
6. `cargo package --list --allow-dirty --locked --offline --no-verify` remains
   exactly 1,300 files. No file is added. The existing Python contract is 532
   lines before this change, and no Rust source or pinned 1,000-line source-size
   rail is touched.

## Dependencies

None.

## Review

- Design review: **ACCEPT** after clarifying five passages, correcting the
  withdrawn-ticket provenance, and making the generic marker and scoped
  structural-path checks precise.
- Code review: **ACCEPT with no findings**. Independent review confirmed the
  five rewrites preserve their evidence and rationale, the living-root scan is
  case-insensitive and avoids false positives, exclusions remain untouched,
  and focused tests, package count, file scope, and diff hygiene pass.

## Implementation evidence

- Red: the new current-Markdown contract failed with exactly five violations:
  the two structural-variant passages, the two external-capture citations, and
  the article-download consumer reference.
- Green: the focused documentation-consistency contract passes all 16 tests
  after replacing those references with self-contained descriptions.
- Repository gates: `make lint` and `make spec` pass. A complete `make test`
  rerun passes 3,147 Rust tests (30 skipped), 893 Python contracts (3 skipped),
  and the strict documentation build. The first run encountered one unrelated
  timing-sensitive article-asset test; that exact test and the full rerun both
  passed without source changes. The Cargo package remains exactly 1,300 files,
  and `git diff --check` passes.

## Completed 2026-09-05

Five stale private-project passages in living BioMCP reports and open issues
now describe the relevant evidence and rationale without naming unavailable
external projects or paths. A generic case-insensitive static audit covers all
Markdown below `docs/`, `architecture/`, and `sdlc/issues/`, while preserving
legitimate provider, public-repository, historical, fixture, onboarding, and
reproduced path-leak references.

Primary verification passed after independent review: `make lint`; `make test`
(3,147 Rust tests passed with 30 skipped, 893 Python tests passed with 3
skipped, and strict documentation passed); and `make spec` (all routine groups,
including 140 serialized cases with 4 skipped, 39 parallel-isolation cases,
and 8 static cases). The package remains exactly 1,300 files and `git diff
--check` passes. The first broad test attempt had one unrelated timing failure;
that exact test and the complete rerun passed without source changes.
