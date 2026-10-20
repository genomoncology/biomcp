---
flow: build
priority: 2
deps: [2098]
---

# 2106: Find the one failing Rust test on the macOS build host

Status: OPEN.

Milestone: 1.0

## Outcome

- Reproduce on the macOS build host, name the test, and decide: macOS-specific (fix or refuse on macOS with a recorded reason) or a real breakage (fix on the line).

## Evidence

- Proof: the existing acceptance cases and their stated gate checks below.

- 2098's acceptance run on the macOS build host (line base 72c81e535 + gate-host branch at 7bad55dfb): `make test` fails exactly one Rust test. Unknown whether macOS-specific or a tip breakage; 2098 has no Rust changes, so the failure predates or is independent of its portability layer. Fail-fast stopped the run before the pytest contracts.

### Acceptance

1. `make test` on the macOS build host reaches the pytest contracts with zero Rust failures, or each macOS-specific refusal is recorded with its reason.
