---
flow: build
priority: 8
---
# Reject symlinks at saved fulltext paths

Saved article fulltext uses a predictable managed-cache filename. When that
path already exists, the download helper follows it with ordinary metadata and
accepts it whenever the resolved target is a file. Managed permission repair
intentionally skips symlinks, while later outline and line-range reads follow
the same link. A same-account process or inherited unsafe cache can therefore
substitute another text file for BioMCP's saved article.

A managed fulltext entry must be a regular, single-linked file inside the
managed download directory at the point BioMCP accepts and reads it. Reject a
symlink or replacement race; do not repair or follow it. Preserve the current
private atomic-write behavior for legitimate files on Unix and Windows.

## Done when

- A symlink at the expected saved-fulltext path is rejected before cached
  content is accepted or read.
- Replacing a validated path with a symlink cannot redirect the subsequent
  outline, line-range, or summary read.
- Legitimate existing files are still repaired to private permissions and
  reused, while non-regular or multiply linked entries fail clearly.
- Regression tests use only temporary managed-cache roots and do not depend on
  public providers.

## Authorized test changes

Design may restate saved-download and filesystem-entry assertions in
`src/utils/download.rs`, managed privacy assertions in `src/cache/private.rs`,
and fulltext read assertions in `src/cli/article/fulltext_view.rs`.
