---
priority: 2
---

# 2307 — Restrict routine live advisory checks to dependency changes

Status: OPEN.

Milestone: 0.9.4

Depends on: 2305

## Outcome

The landing gate runs live dependency advisory checks only when the candidate changes Cargo.lock, Cargo.toml, uv.lock or pyproject.toml. Every full main check and every release qualification still runs live advisory checks, including when no dependency input changes. A newly published advisory can block release admission. Preserve the remaining lint, license, test and specification checks.

Changes: `Makefile`, `bin/lint`, `tests/test_lint.py`, `tests/test_dependency_advisory_gate.py`, `AGENTS.md`, `CHANGELOG.md`, `sdlc/records/2307-dependency-advisory-gate.md`.

## Keeps

Use the existing gate commands and dependency-input source of truth. Keep advisory failures visible when required. Missing or ambiguous comparison information fails safely or runs the check; it must never silently suppress required checks. Do not disable certificate checks, suppress advisories, weaken license policy or add a second advisory implementation.

## Reproduction before code

The text-only 2284 landing ran cargo-deny advisories on an unchanged lockfile and failed under newly published RUSTSEC-2026-0382 and RUSTSEC-2026-0381. Retained job 2284/1792524739616-64224 and record 2305 preserve the rejected lock, advisory database and subsequent minimal repair. The developer must reproduce the current unconditional gate selection on repaired main with the exact command and output before changing it. No new vulnerable dependency installation is needed.

## Proof

An outside-in command control shows routine landing with unchanged dependency inputs skips only live advisory work, while a change to each supported dependency input requires it. Missing comparison context must not skip the check. Full main and release entry points require advisory work with unchanged inputs. Required advisory failure propagates; remaining lint and license checks still run. Execute affected controls and the routine suite on the branch merged with current main before fresh review.

## Order

PM authorizes this small 0.9.4 change alongside the reliability lanes. The coordinator batches it when existing gate tooling permits; gate-definition isolation rules remain enforceable. The actual resulting main requires its full check before release admission.
