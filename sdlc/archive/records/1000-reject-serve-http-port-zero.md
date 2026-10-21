---
flow: quickfix
priority: 9
---

# Reject serve-http port zero

`serve-http --port 0` currently starts on an undisclosed operating-system port while reporting port zero. Reject zero as invalid usage before binding and advertise the supported range as 1–65535. Normal, occupied, and Host-validation behavior remains unchanged.

Red-green coverage belongs in `src/cli/system/tests.rs` and `tests/test_mcp_http_surface.py`; their port validation expectations may be expanded.
