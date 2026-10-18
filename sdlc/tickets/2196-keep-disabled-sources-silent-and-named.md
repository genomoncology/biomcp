---
priority: 2
---

# 2196 — Keep disabled sources silent and named

Status: OPEN.

Milestone: 0.9.3

## Outcome

A disabled source is never contacted, including by `biomcp health`, and every place it would have appeared names the setting instead of a failure.

- Evidence: release review of 7025e117 with all sources disabled: `biomcp health` still contacts MyGene, MyVariant, MyChem, Monarch, HPO, MyDisease, HPA and the FDA orphan site, then lists them as never called. Disabled ClinVar, gnomAD and CIViC sections say "request failed", "temporarily unavailable" or "Retry". CIViC and ClinVar data carried inside MyVariant still appear, contrary to the changelog's "its carried fields drop". Disabled article sources show `"enabled": true, "status": "degraded"`, `complete: false` and WARN lines. The MCP override error blames the environment variable when the flag set it. The `commercial-restricted` group omits Cancer Genome Interpreter, whose recorded terms restrict commercial use, because it lacks the commercial terms field; check DisGeNET and AlphaGenome the same way.
- Cause: health builds its own client (`src/cli/health/runner.rs` near 344), matches rows by display name, and the FDA orphan probe uses a raw client.
- Changes: route health probes through the disabled-source check by source id; report disabled sections and article sources as disabled with the setting's name; drop carried fields of disabled sources; name the actual origin of an override; fix the terms records so the group matches its documented rule.
- Proof: traced-network outside-in cases: health with sources disabled makes no call to them; each named section reports disabled; carried fields absent; group membership matches the recorded terms.

## Progress

- 2026-10-18 started
