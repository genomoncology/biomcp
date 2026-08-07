---
base: 199e2c973355b2f65ae31777e8746fbd22f6ed1d
head: c28fd7f114a4a5f4a1c96236cf771c500c986296
---
BioMCP intentionally supports `make spec BIOMCP_BIN=/path/to/executable` so March, release checks, and operators can test an already-built artifact without rebuilding it. The product scenarios use that artifact successfully, but three Makefile meta-contracts fail because command-line `BIOMCP_BIN` and related overrides propagate through recursive `make` via `MAKEFLAGS`/`MAKEOVERRIDES`; `env -u BIOMCP_BIN` alone does not restore the default dry-run contract. A supported gate mode should finish green rather than require humans to dismiss false failures.

Imported from March ticket 509. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/509-make-caller-provided-binary-spec-mode-pass-cleanly
