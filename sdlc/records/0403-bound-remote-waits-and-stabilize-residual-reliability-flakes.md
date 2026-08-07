---
base: e0023ea759f63b5c23213daf04870f1923f812ba
head: 0b8da0dad44014794436837f1337d54613d2882f
---
Several remaining issues are reliability/performance boundaries where the right outcome is an explicit policy plus an automated check: extreme `Retry-After` headers can stall CLI commands, study downloads need a no-stall timeout contract, warm/performance canaries have intermittent outliers, and live-source flakes should not silently weaken release confidence.

Imported from March ticket 403. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/403-bound-remote-waits-and-stabilize-residual-reliability-flakes
