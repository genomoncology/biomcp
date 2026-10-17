---
priority: 2
---

# 2179 — Accept a bare output filename

Status: OPEN.

Milestone: 0.9.3

## Outcome

`get article ID asset NAME --output bare.pdf` writes `bare.pdf` in the current directory, the same as `./bare.pdf`.

- Evidence: research's 0.9.3 spot check on 2026-10-17 hit "Output directory does not exist:" with an empty directory. `create_unique_sibling_temp` in `src/utils/download.rs` takes `path.parent()`, which is an empty path for a bare filename, and `"".is_dir()` is false.
- Keeps: refusing a real missing directory, and managed-tree security.
- Changes: claims `src/utils/download.rs` `CHANGELOG.md`. Treat an empty parent as the current directory.
- Proof: an outside-in CLI test against the recorded-reply server writes `bare.pdf` from a bare name; on current main it fails with the message above. A missing directory still fails.
