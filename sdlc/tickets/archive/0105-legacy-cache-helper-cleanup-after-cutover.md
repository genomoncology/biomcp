---
flow: build
priority: 5
---
# Legacy cache-helper cleanup after cutover

T104 (runtime cache-root cutover and hermetic proof) cuts all live callers over to `resolve_cache_config()` but intentionally leaves the legacy helper functions (`biomcp_cache_dir()`, `biomcp_downloads_dir()`, `cache_path()`) in place so the cutover ships cleanly. This ticket removes those now-dead helpers and trims any associated dead imports and tests once T104 is merged.

Completed under March on 2026-04-01, as March ticket 105. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/105-legacy-cache-helper-cleanup-after-cutover
