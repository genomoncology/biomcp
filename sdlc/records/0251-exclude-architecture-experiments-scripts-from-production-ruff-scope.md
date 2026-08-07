---
base: ccad7500dc11579641d918497e22368c65ad7bd0
head: 84cfe22bed23a7c90de24d8196a5af4919e1965a
---
Two consecutive spike tickets (243 and 244) failed at `verify+merge` on trivial ruff lint in `architecture/experiments/{slug}/scripts/` — one on E731 (lambda assignment), one on F401 (unused import). Both scripts passed the spike flow's in-flow baseline but tripped the full `make check` at merge time, which stopped the queue, auto-paused the team, and required manual recovery. Production lint rules should not apply to scratch experiment scripts; the fix belongs in the ruff config, not in every spike's harden step.

Imported from March ticket 251. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/251-exclude-architecture-experiments-scripts-from-production-ruff-scope
