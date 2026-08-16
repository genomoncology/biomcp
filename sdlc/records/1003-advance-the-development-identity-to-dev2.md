---
flow: quickfix
priority: 8
---

# Advance the development identity to dev.2

After this correction batch, commit the private package identity as Rust `0.9.0-dev.2` and Python `0.9.0.dev2`. Locks, current-version documentation, and repository identity tests move together. Public release metadata remains exactly `0.8.25`; no tag, candidate staging, signing-policy change, or publication belongs to this ticket.

Restatements are authorized in `architecture/technical/overview.md`, `docs/reference/release-process.md`, `spec/surface/mcp.md`, `tests/test_version_sync_script.py`, `tests/test_citation_contract.py`, `tests/test_directory_submission_contract.py`, and the root version lock files.
