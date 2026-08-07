---
flow: build
priority: 5
---
# Make caller-provided binary spec mode pass cleanly

BioMCP intentionally supports `make spec BIOMCP_BIN=/path/to/executable` so March, release checks, and operators can test an already-built artifact without rebuilding it. The product scenarios use that artifact successfully, but three Makefile meta-contracts fail because command-line `BIOMCP_BIN` and related overrides propagate through recursive `make` via `MAKEFLAGS`/`MAKEOVERRIDES`; `env -u BIOMCP_BIN` alone does not restore the default dry-run contract. A supported gate mode should finish green rather than require humans to dismiss false failures.

Completed under March on 2026-07-13, as March ticket 509. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/509-make-caller-provided-binary-spec-mode-pass-cleanly
