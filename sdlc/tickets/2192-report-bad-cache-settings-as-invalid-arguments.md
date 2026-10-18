---
priority: 1
---

# 2192 — Report bad cache settings as invalid arguments

Status: OPEN.

Milestone: 0.9.3

## Outcome

A bad cache setting fails with exit 2 and an invalid-argument message naming the setting and value, as v0.9.2 does, on the CLI, in JSON and over MCP.

- Evidence: release review of 7025e117: `BIOMCP_CACHE_MAX_SIZE=10GB biomcp get gene NRAS` exits 1 with `HTTP request to MyGene.info failed. Retry the remote source.` (JSON code `http_middleware`) while `biomcp health` shows every API ok. `BIOMCP_CACHE_MIN_DISK_FREE=abc`, `BIOMCP_CACHE_MAX_AGE=-1` and `max_size = "10GB"` in `cache.toml` do the same. v0.9.2 exits 2 with `Invalid argument: BIOMCP_CACHE_MAX_SIZE must be a positive integer number of bytes: got '10GB'` and names toml parse errors.
- Cause: 2173's lazy shared cache resolves configuration inside the request middleware (`resolve_http_cache_config()?` near `src/sources/mod.rs:608`, mapped near 650-654), so a configuration error becomes a request error. 2172's fall-through must not swallow it either.
- Changes: resolve and validate cache configuration once before the first request and report it as invalid input; keep cache I/O failures falling through to the provider.
- Proof: outside-in cases for each setting above, CLI exit code and message, JSON code, and one MCP reply; red on current main.

## Progress

- 2026-10-18 started
