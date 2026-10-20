# 2311 — Declared focused Python tools

Supply pytest and the exact required Ruff from one declared bootstrap in each job environment. Keep the Ruff pin in one source and preserve version and command failures. Never obtain production tools from another ticket environment or a global PATH fallback.

Before source edits, Gator 2311/1792532482401-1499961 on published a5314c569 used a fresh UV_PROJECT_ENVIRONMENT under its own checkout and ran make sync-python-dev. That exact environment supplies pytest 9.0.3, then its Python -m ruff fails No module named ruff with exit 1. This direct module probe does not fall back to host PATH. The existing 2299 root cause record and the bounded repeated-setup RCA describe the incomplete environment class; the separate 2300 tracked-text permission error is not a Ruff absence.

Planned source is the existing Python dev dependency and lock, sync-python-dev target, lint entry point, lint command controls, focused-check instructions and changelog. The accepted 2308 security branch is imported as the declared dependency to preserve its patched locks. The narrow convention owns the checkout .venv and invokes its Python modules for pytest and Ruff, with an explicit job-local UV environment for existing Make commands. Derive the required Ruff version from its single exact declaration. No extra wrapper, release gate or retained-evidence rewrite is planned.

Focused fresh-environment, foreign-environment, version and failure controls, required routine qualification and fresh review remain pending. Raw belongs under the existing experiment 2102-biomcp-2258-final-live-replay/python-tools/2311; final command paths and results must bind their actual source.

## Focused source and workflow consumer proof

Final candidate f636325da9e4c432252333ca993642a597d2fbb0 includes published security main 3c9ddc151. It declares the exact Ruff pin once in the dev dependencies, derives lint version checks from that declaration and uses job-local .venv Python modules. make lint depends on the existing sync-python-dev bootstrap, with explicit local UV_PROJECT_ENVIRONMENT. Global or foreign environment tools do not supply required checks.

Gator 2311/1792532620090-1514313 passed forty lint controls and real bootstrap/tool probes. Expanded M5 2311/1792532668836-9535 passed forty-three controls, Ruff, Bash syntax and tracked text, including missing and duplicate pins, wrong and failed versions and hostile foreign environments. M5 uv retained the minimal lock unchanged; the two unrelated metadata rewrites from the newer Gator uv were reverted before the final candidate. The only lock additions beyond accepted 2308 are Ruff and its root dev edges.

The real duplicate workflow Ruff pin and lane setup global install are removed; the reusable lint lane reaches the same bootstrap through make lint. No trigger, job or release gate is added. The affected workflow contract and changed Python lint passed Gator 2311/1792532859017-1534541. Final M5 routine 2311/1792532844466-40234 is running on exact f636; earlier obsolete waiters were verified canceled before execution. Fresh review and the actual routine result remain required. Canonical raw is python-tools/2311 in the owning admission experiment.
