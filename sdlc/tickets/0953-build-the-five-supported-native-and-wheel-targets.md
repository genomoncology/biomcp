---
flow: build
priority: 7
deps: ["0937", "0939", "0940", "0952", "0958"]
---
# Build the five native and wheel targets

Native archives and PyPI wheels need one explicit platform contract. The
current wheel matrix omits Linux ARM64, and release readiness cannot be inferred
from a YAML matrix that never proves the produced executable.

## Artifact contract

Retain and reverify ticket 0952's exact Linux x86_64 archive and wheel bytes,
then register the other four platform builders so one stage run contains
native archives and `biomcp-cli` wheels for exactly:

- Linux x86_64, glibc 2.28;
- Linux ARM64, glibc 2.28;
- macOS x86_64;
- macOS ARM64; and
- Windows x86_64.

The support floors and evidence are part of the artifact name and
documentation. Both macOS architectures require macOS 14.0 or later, set and
inspect deployment target 14.0, and execute on the standard pinned
`macos-15-intel` and `macos-15` GitHub-hosted runner images. Mach-O load
commands and imported symbols must prove that building on 15 did not raise the
14.0 compatibility floor. Windows x86_64
uses Rust's Tier 1 `x86_64-pc-windows-msvc` contract: Windows 10 or later for
clients and Windows Server 2016 or later for servers. Execute it on the
standard pinned `windows-2022` image with a pinned Rust toolchain, Visual
Studio toolset, and Windows SDK; inspect its PE headers and imports so a
dependency cannot silently raise that floor. These lanes must not provision a
paid, larger, or self-hosted runner. Automated CLI/process evidence is not a
claim that CI drove the Claude Desktop user interface.
Linux's glibc 2.28 floor is an ABI contract, not a promise that an end-of-life
distribution remains supported. A future wider/older OS claim requires its own
oldest-runtime proof.

Every target comes from ticket 0952's one full SHA/version and enables the same
public default features including PNG. Each native archive contains only the
full `biomcp` executable. Each wheel contains that executable plus the small
0939 compatibility alias; it never contains two full application binaries.
Every artifact carries a checksum and provenance entry. The macOS and Windows
jobs compile each target exactly once, record the private unsigned intermediate
hash, and pass every shipped executable through 0958 before either package is
assembled. That includes separate finalization and evidence for the full
`biomcp` program and 0939's executable wheel shim. Each native archive and wheel
is assembled once from the resulting signed bytes. An unsigned intermediate is
never uploaded as a candidate artifact, and archive or wheel assembly may not
sign, strip, or mutate its executable. Linux builds use a digest-pinned
manylinux_2_28-equivalent environment; symbol inspection rejects any imported
GLIBC version above 2.28 and a pinned oldest-runtime smoke executes the result.

## Done when

- Each target's isolated native archive and wheel install runs `version`, help,
  JSON success/error, local stdio MCP initialize/tools-list, and fixture-backed
  PNG signature checks without reading a source checkout.
- Load-command, platform-tag, oldest-runtime, and Windows PE/import checks fail
  when an artifact raises the stated floor or advertises an unproved target.
- The protected macOS and Windows lanes require 0958's real finalization
  evidence. They verify the signed executable hash after archive extraction and
  wheel installation; fixture-only or unsigned identities cannot satisfy a
  candidate run.
- Wheel tags, bundled libraries, executable count/size, license/SBOM contents,
  and absence of test fixtures/source/cache/secrets are inspected.
- Ticket 0934's canonical gates run on every main push. The expensive five-
  platform artifact matrix runs only inside ticket 0952's explicitly invoked
  `stage` for a selected full main SHA and uploads one write-once, checksummed
  manifest of hashes keyed by run ID; local Linux proof and workflow structure
  are blocking for this implementation flight.
- The Linux x86_64 job invokes 0952's registered builder rather than a second
  implementation; a counting contract fails if any artifact is built twice in
  one stage run or an upstream hash is replaced.
- The release promotion gate in 0957 requires a successful candidate run for
  all five jobs from that exact stage run at the release SHA. Workflow text
  alone is never reported as cross-platform execution evidence.
- No artifact is published by this ticket.

## Authorized test changes

Design commits may restate native/wheel build matrices, pinned tool setup,
archive and maturin packaging, 0958 integration, artifact inspection, local
fixture smoke, and platform support-floor documentation. Product behavior must
remain identical across targets.

The src line ceiling may not rise.
