# 2311 — Declared focused Python tools

Supply pytest and the exact required Ruff from one declared bootstrap in each job environment. Keep the Ruff pin in one source and preserve version and command failures. Never obtain production tools from another ticket environment or a global PATH fallback.

Before source edits, Gator 2311/1792532482401-1499961 on published a5314c569 used a fresh UV_PROJECT_ENVIRONMENT under its own checkout and ran make sync-python-dev. That exact environment supplies pytest 9.0.3, then its Python -m ruff fails No module named ruff with exit 1. This direct module probe does not fall back to host PATH. The existing 2299 root cause record and the bounded repeated-setup RCA describe the incomplete environment class; the separate 2300 tracked-text permission error is not a Ruff absence.

Planned source is the existing Python dev dependency and lock, sync-python-dev target, lint entry point, lint command controls, focused-check instructions and changelog. The accepted 2308 security branch is imported as the declared dependency to preserve its patched locks. The narrow convention owns the checkout .venv and invokes its Python modules for pytest and Ruff, with an explicit job-local UV environment for existing Make commands. Derive the required Ruff version from its single exact declaration. No extra wrapper, release gate or retained-evidence rewrite is planned.

Focused fresh-environment, foreign-environment, version and failure controls, required routine qualification and fresh review remain pending. Raw belongs under the existing experiment 2102-biomcp-2258-final-live-replay/focused-python/2311; final command paths and results must bind their actual source.
