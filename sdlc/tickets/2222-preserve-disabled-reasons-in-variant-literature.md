---
priority: 2
---

# 2222 — Preserve disabled reasons in variant literature

Status: OPEN.

Milestone: 0.9.4

## Outcome

Variant article search names configured source disables consistently through CLI and MCP. Disabled resolution providers do not become unexplained provider failures. Preserve the established variant identity rules.

- Evidence:2204 correction0002 at3dc15da invoked MCP variant_articles with all81 registry IDs disabled and arguments items:[{gene:BRAF,protein:V600E}],limit:1. There were zero traced requests, but isError:true, complete:false,truncated:true and source_status resolution/myvariant status:unavailable,reason_codes:[provider_error]. Error text said variant article sources were unavailable and named no disable setting. Only MCP was observed; CLI needs reproduction.
- Changes: `src/entities/variant/search/mod.rs` `src/entities/article/variant_search.rs` `CHANGELOG.md`. Preserve disabled-source outcomes through existing shared resolution/status mapping; declare the outside-in test path before editing it. No clinical identity or alias-policy change.
- Proof: reproduce the captured MCP case and equivalent CLI case with isolated recorded replies, then assert both name the disable setting/source while making zero blocked-provider requests. Keep genuine provider failures distinguishable. Use2204's retained observation and bounded fixture helpers, without new live captures.
