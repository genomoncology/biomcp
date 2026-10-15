---
flow: build
priority: 2
deps: [2098]
---

# 2106: Find the one failing Rust test on the M5 gate host

Status: OPEN.

Milestone: 1.0

## Evidence

- 2098's acceptance run on M5 (line base 72c81e535 + ticket/2098-m5-gate-host at 7bad55dfb): `make test` fails exactly one Rust test. Unknown whether macOS-specific or a tip breakage; 2098 has no Rust changes, so the failure predates or is independent of its portability layer. Fail-fast stopped the run before the pytest contracts.

## Work

- Reproduce on M5, name the test, and decide: macOS-specific (fix or refuse on macOS with a recorded reason) or a real breakage (fix on the line).

## Acceptance

1. `make test` on M5 reaches the pytest contracts with zero Rust failures, or each macOS-specific refusal is recorded with its reason.
