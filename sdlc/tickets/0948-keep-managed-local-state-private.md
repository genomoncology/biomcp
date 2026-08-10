---
flow: build
priority: 9
deps: ["0935"]
---
# Keep managed local state private

Article sessions contain query terms and article identifiers, while the HTTP
cache contains provider responses. Their managed directories and files need a
deliberate local access boundary independent of the retention lifetime.

## Filesystem contract

On Unix, every BioMCP-created cache/session directory is mode 0700 and every
regular metadata/data file is 0600. Atomic temporary files start private rather
than being narrowed only after content is written. When BioMCP opens an
existing managed root it narrows overly broad permissions on managed paths,
but never changes an unrelated ancestor or follows a symlink.

On Windows, create the managed root with access limited to the current user
using the platform-supported user-private mechanism. If BioMCP cannot establish
or verify that boundary, return a typed local-storage error before writing
query or response content; do not silently continue with broad access.

## Done when

- Permission tests cover initial directory, metadata, blob, session, lock, and
  atomic-temp creation using a deliberately permissive process umask.
- Reopening an overly broad managed path narrows only managed entries and leaves
  parent sentinels, symlinks, hard-link edge cases, and unrelated files intact.
- Failure to secure a new store writes no sensitive content and preserves an
  existing valid store.
- Unix and Windows CI exercise their platform contract; unsupported permission
  APIs do not become silent skips on a supported release target.
- Policy and cache/session documentation state that local access controls do
  not govern upstream provider retention.

## Authorized test changes

Design commits may restate cache/session creation, atomic write, lock,
permission, platform, and policy tests in `src/cache` and
`src/cli/article/session.rs`. Existing symlink/root confinement, cleanup,
concurrency, and retention assertions remain covered.

The src line ceiling may rise by at most 180 lines.
