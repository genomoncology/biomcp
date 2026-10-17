---
priority: 2
---

# 2179 — Accept a bare output filename

Status: OPEN.

Milestone: 0.9.3

Reviews: revision be2dfea, accept

## Outcome

`get article ID asset NAME --output bare.pdf` writes `bare.pdf` in the current directory, the same as `./bare.pdf`.

- Evidence: research's 0.9.3 spot check on 2026-10-17 hit "Output directory does not exist:" with an empty directory. `create_unique_sibling_temp` in `src/utils/download.rs` takes `path.parent()`, which is an empty path for a bare filename, and `"".is_dir()` is false.
- Keeps: refusing a real missing directory, and managed-tree security.
- Changes: claims `src/utils/download.rs` `tests/article_asset_terminal.rs` `CHANGELOG.md`. Treat an empty parent as the current directory.
- Proof: an outside-in CLI test against the recorded-reply server writes `bare.pdf` from a bare name; on current main it fails with the message above. A missing directory still fails.

Coordinator outcome before code: Write a bare asset output filename in the current directory exactly as ./filename, while refusing genuinely missing directories and preserving managed-tree security. Before product edits, reproduce on current main through the build job and report the exact command/output, proposed implementation/test files and the outside-in failing proof. Root records that evidence and scope before implementation. Developers write no ticket or status records.

## Progress

- 2026-10-17 started


## Coordinator reproduction and scope

Qualified CLI source `926b5786d400d3ec8b54c929701c1e46c62b5a95` differs from current main `90c00c371` only by ticket records. CLI SHA256: `e9992fadcb93e290a6c310827a14368444dcd1305784661c659c46311ed9083d`.

Outcome: a bare output filename writes in the current directory, preserving genuine missing-directory refusals and managed-tree security. Add `tests/article_asset_terminal.rs` to claims; extend existing `binary_article_assets_protect_terminals_and_preserve_files`.

Job 2179/1792273495965-71890 ran `biomcp get article 22663011 asset traces-s1.csv --output bare.pdf` against recorded replies: exit1, `Error: IO error: Output directory does not exist:` with empty parent. Explicit `./explicit.pdf` succeeded with recorded CSV bytes. Job 2179/1792273570087-81684 failed the actual CLI assertion requiring bare-name exit0. Authorize that acceptance extension and the minimal shared parent-path fix; retain security controls.

Coordinator verification: GREEN2179/1792273896874-3242 passed:12 selected asset/download tests, fmt, targeted Clippy with warnings denied and quality ratchet. The applied product patch matches local sourceb139ad12b537a2b23c8ab56a6d85ee9a8e5aac7a. No existing runtime comparison scenario exercises this output path. Fresh review follows; no duplicate development run is required.
