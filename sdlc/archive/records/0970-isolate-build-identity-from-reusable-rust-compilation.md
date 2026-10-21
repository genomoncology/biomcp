---
base: 80715b7f3ab829076a8449a0b88649b072982f74
head: 4622d13499d52356c6213925a06d777560459fbc
---

Moved Git-derived version identity out of the package build script and into a
small command wrapper whose compile inputs are consumed only by the thin
executables. The reusable library and unit tests now remain fresh when only
Git `HEAD` moves. Routine, lint, release, install, and CI build paths use the
wrapper; Git-free archives report package version plus explicit unknown
provenance and ignore unrelated surrounding repositories.

Synthetic Cargo tests prove HEAD-only, executable-source, and library-source
artifact ownership, exact tags, dirty tracked source, and Git-free archives.
On BioMCP, two clean HEAD-only builds reported a fresh `biomcp_cli` library and
a rebuilt `biomcp` binary, taking 2.59 and 1.79 seconds. The latter is roughly
36x faster than the prior 65.26-second optimized invalidation. Same-HEAD warm
was 0.38 seconds; a fresh-target development build was 102.76 seconds.

Full lint passed before the implementation commit. The complete Rust lane
passed 2,835 tests, and after correcting two affected test harnesses the full
Python lane passed all 484 tests in 105.41 seconds.
