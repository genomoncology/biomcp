---
base: 07f5fd12cf4fdab7e9a68364019b91ce8e90b6be
head: f121df4916ae5ec116b5b9c47bf098c9b2bc7efa
---
T104 (runtime cache-root cutover and hermetic proof) cuts all live callers over to `resolve_cache_config()` but intentionally leaves the legacy helper functions (`biomcp_cache_dir()`, `biomcp_downloads_dir()`, `cache_path()`) in place so the cutover ships cleanly. This ticket removes those now-dead helpers and trims any associated dead imports and tests once T104 is merged.

Imported from March ticket 105. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/105-legacy-cache-helper-cleanup-after-cutover
