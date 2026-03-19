# BioMCP Strategy

## What We Are Building

BioMCP is a single-binary Rust CLI and MCP server that gives AI agents and
researchers read access to biomedical databases through a unified command
grammar. It is open-source (MIT), distributed via PyPI and direct binary
install, and maintained at biomcp.org.

## Architectural Commitments

- **Single binary.** One `biomcp` binary speaks both CLI and MCP server modes.
  Adding a new entity does not require a new process.
- **Rust core, Python packaging.** The Rust binary is wrapped by `biomcp-cli`
  on PyPI. Python is packaging only, not logic.
- **Federated reads, no writes.** BioMCP queries upstream APIs. It never writes
  to external systems or modifies upstream data.
- **Rate limiting is process-local.** A shared `biomcp serve-http` endpoint is
  the canonical scaling answer for multi-worker deployments.
- **Progressive disclosure.** Default output is a concise summary card.
  Sections are additive and source-aware; requested sections must explain
  unsupported, empty, or unavailable states truthfully.
- **Cross-entity pivots over rebuilding filters.** Helper commands (`variant
  trials`, `gene drugs`, etc.) let users move between related entities without
  repeating query context.

## Quality Bar

- Skills must produce correct, well-formatted output for common biomedical
  workflows (variant, trial, drug, article).
- CLI help, error messages, and suggested next steps must be accurate,
  source-aware, and not reference stale or unsupported commands.
- Evidence URLs must be present in output and reachable.
- New source work is not done until specs and operator checks intentionally
  cover the shipped surface (`biomcp health`, contract-smoke when suitable).
- `search all` is the unified entry point and must work reliably across all
  entities.
- Release artifacts: Cargo binary, PyPI wheel (`biomcp-cli`), binary installer
  (`install.sh`).

## Active Goals

- **G002 — Give Back to the Community:** Real people using BioMCP for real
  biomedical work. Quality of the tool IS the contribution.
- **G003 — Ship v1.0:** Skills, CLI, docs, and tests meet a standard you'd
  hand to a colleague. Paper and citation published.

## Repo Analysis Sources

- `design/functional/overview.md` — what BioMCP does and who it serves
- `design/technical/overview.md` — system shape, build, runtime constraints
- `design/ux/cli-reference.md` — command grammar and demo workflows
