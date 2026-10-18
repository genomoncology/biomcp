---
priority: 2
---

# 2222 — Preserve disabled reasons in variant literature

Status: OPEN.

Milestone: 0.9.4

## Outcome

Variant article search names configured source disables consistently through CLI and MCP. Disabled resolution providers do not become unexplained provider failures. Preserve the established variant identity rules.

- Evidence:2204 correction0002 at3dc15da invoked MCP variant_articles with all81 registry IDs disabled and arguments items:[{gene:BRAF,protein:V600E}],limit:1. There were zero traced requests, but isError:true, complete:false,truncated:true and source_status resolution/myvariant status:unavailable,reason_codes:[provider_error]. Error text said variant article sources were unavailable and named no disable setting. Only MCP was observed; CLI needs reproduction.
- Changes: `tests/test_variant_article_disabled_sources.py` `src/entities/variant/search/mod.rs` `src/entities/article/variant_search.rs` `CHANGELOG.md`. Preserve disabled-source outcomes through existing shared resolution/status mapping; declare the outside-in test path before editing it. No clinical identity or alias-policy change.
- Proof: reproduce the captured MCP case and equivalent CLI case with isolated recorded replies, then assert both name the disable setting/source while making zero blocked-provider requests. Keep genuine provider failures distinguishable. Use2204's retained observation and bounded fixture helpers, without new live captures.

## Progress

- 2026-10-18 started

Pre-code plan: preserve configured source-disable reasons through the shared variant-literature event and status mapping. New real CLI and MCP cases use existing isolated replay/RawStdioMcp helpers, all IDs disabled and only myvariant-info disabled, and assert source_disabled with the setting named and zero blocked-provider requests. Reuse existing sources::disable::unavailable_disabled_reason; retain only recognized configuration detail, preserving genuine provider-error distinctions and identity rules. Existing resolution call sites already use record_error, so no resolution-module change is expected. Actual current-main RED precedes the narrow product fix; execution waits for coordinator admission.

Actual current-main RED: job2222/1792355284327-34971 copied and verified87d53c CLI (SHA25631df21871fbf9ca95580f2ac15a05d3bb8f53079672db177e5363b6a9d611a8f). Four CLI/MCP cases with all IDs disabled or only myvariant-info disabled failed the intended reason_codes assertion: resolution/myvariant unavailable with provider_error and generic detail, without the setting. Zero replay requests, empty results, complete:false and truncated:true assertions already passed. Cases took0.64s, job1.791s. Proceed with recognized disabled-reason propagation through shared event/status/outage mapping; GREEN waits for the released slot.
