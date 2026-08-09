# Cache-clear Unix-socket test fails when TMPDIR has a long path

Severity: should-fix. The routine Rust test gate can fail before exercising the
cache-clear behavior when its temporary directory is rooted in a long CI or
agent-provided `TMPDIR`.

`cache::clear::tests::clear_rejects_special_file_before_mutation` creates a
Unix socket at `<TMPDIR>/biomcp-test-special-file-*/http/special.sock`. Unix
socket paths are capped by `SUN_LEN`; with the flight runtime's long `TMPDIR`,
`UnixListener::bind` panics with `path must be shorter than SUN_LEN`.

First observed while reproducing ticket 0875 on 2026-08-09. The same gate
passes with `TMPDIR` rooted at `/tmp`, so the failure is unrelated to the
cache-clear behavior or that ticket's version fix.

## Fix shape

Use a short, test-owned temporary root for this Unix-socket fixture, or make
the shared `TempDirGuard` select a short root where a test needs Unix-domain
sockets. Preserve the test's special-file rejection assertion.
