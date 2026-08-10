---
flow: publish
priority: 1
---
# Release v0.8.26 from one verified commit across every distribution channel

Carried over from March ticket 499 when BioMCP moved to the sdlc
factory. The body below is March's, unchanged; it was already written to
stand alone. Work products from any earlier attempt:

    /home/ian/workspace/planning/biomcp/artifacts/499-release-v0-8-26-from-one-pinned-commit-after-audit-cleanup

Held as a draft: this is a release, and the assembly has no publish
flow yet. Promote it by moving it up one directory once one exists.
## Why

BioMCP needs one trustworthy v0.8.26 release boundary. The latest public version,
v0.8.25, has a known provenance defect: the published Linux archive and PyPI-installed
binary identify commit `a6694289`, while the immutable tag resolves to a different
commit. The current release workflow now resolves and checks a requested tag, but it is
still triggered by an already-public GitHub Release, so a release can be visible before
the distribution jobs succeed. That happened for v0.8.25: the release was public while
several publication runs failed.

The source history is also mislabeled. Current post-tag work is still recorded under the
v0.8.25 changelog heading, and a routine test requires the absence of an `Unreleased`
section. The intended changelog repair ticket was superseded without a replacement.

Finally, the existing distribution paths are uneven. Native GitHub assets include Linux
ARM64, but PyPI does not. The GHCR image is Linux AMD64-only, runs as root, and is built
through a separate source-compilation path. Homebrew is published through the existing
`genomoncology/homebrew-biomcp` tap, but the release workflow does not prove a real
post-publish install. MCP Registry and MCPB metadata exist, but BioMCP is not yet in the
official MCP Registry and no installable `.mcpb` asset is released.

This ticket makes v0.8.26 the complete, current release: all work since v0.8.25,
including the completed exact-variant and literature chain 598-609, ClinGen chain
610-616, and audited release repairs 617-623, comes from one pinned commit and is proven
through every supported distribution path. It remains a draft until Ian explicitly
approves the release cut.

## Scope

### 1. Freeze the complete v0.8.26 source boundary

- Begin only after every frontmatter dependency, including repairs 617-623, is complete
  and merged to a clean, up-to-date `main`. Do not cherry-pick a subset of the ClinGen
  or release-repair chain.
- Review the entire commit range from `v0.8.25` through the proposed release commit.
  Curate all user-visible changes; do not limit release notes to the tickets named here.
- Explicitly retain and regression-check the completed strict variant identity and
  literature work from tickets 598-609, including caller-supplied RefSeq identities,
  strict/discovery retrieval provenance, auditable article identity observations, the
  frozen positive/collision/pagination/outage gate, and provider-linked confirmation.
- Include the completed ClinGen contracts from 610-616 and repairs 617-623: ERepo
  assertions, portable typed CSpec source bytes, CAR normalization/equivalence with its
  restored frozen contract, PMID-bound PubTator and optional LDH observations, namespace
  and non-interference gates, content-addressed captures, deterministic routine gates,
  fail-closed installation, and the final unchanged live recall gates.
- Do not add unrelated product behavior during release preparation. A defect found by a
  release gate must be fixed and re-proven before the tag; it must not be waived into the
  release merely because this is a publish ticket.

### 2. Restore changelog truth

- Put exactly one `## [Unreleased]` section above the immutable v0.8.25 section on normal
  post-release `main`.
- Move every post-v0.8.25 entry currently attributed to v0.8.25 into the correct
  v0.8.26/Unreleased history. Never rewrite the published v0.8.25 facts to imply its
  released bytes contained later work.
- Make the release transition deterministic: release preparation turns the curated
  Unreleased body into `## 0.8.26 — <release date>` and immediately restores an empty
  Unreleased section for subsequent work.
- Replace tests that currently require no Unreleased heading with an offline,
  Git-aware regression that fails when post-tag changes are attributed to the latest
  published version. The gate must fail against the current pre-fix changelog.

### 3. Make publication provenance-correct and fail closed

- Prepare one clean release commit on `main`, resolve its full SHA exactly once, and
  build from that SHA before a tag exists. Every source-consuming staging job must
  check out that candidate SHA and assert `HEAD` equality before building, packaging,
  or documenting. After all private artifacts and gates pass and Ian approves the cut,
  create `v0.8.26` as the first public write pointing exactly to the already-staged
  candidate SHA. Promotion jobs must verify the tag resolution and consume the
  content-addressed staged artifacts without rebuilding them.
- The release commit itself must already contain version `0.8.26` in `Cargo.toml`,
  `Cargo.lock`, `pyproject.toml`, the root package entry in `uv.lock`, `manifest.json`,
  `server.json`, and the top-level software record in `CITATION.cff`. The strict semver
  tag must equal every committed version authority. Release jobs may validate those
  files but must never rewrite or repair their versions after checkout.
- Remove the fictitious preferred citation/DOI (`10.5281/zenodo.XXXXXXX`, version 0.9.0)
  from `CITATION.cff` and its ratchet. Keep a truthful v0.8.26 software citation; add a
  preferred paper citation only after a real paper and DOI exist.
- Derive embedded build identity from the explicitly supplied full candidate commit,
  not the workflow event's incidental `GITHUB_SHA`. Add a deterministic regression
  where those SHAs differ and prove the binary metadata uses the candidate commit;
  promotion separately proves the later tag resolves to that same commit.
- Pin every third-party action used by the write-capable release workflow to a full
  immutable commit SHA, with a comment naming the reviewed release. Scope permissions per
  job: validation/build are read-only; `packages: write` only for GHCR; `id-token: write`
  only for PyPI trusted publishing; and `contents: write` only for the jobs that actually
  publish release/docs state.
- Pin the build environment, not only the actions: exact Rust toolchain and targets,
  exact `uv`/maturin/protoc/package-tool versions, and Docker builder/base images by
  immutable digest. Use fixed GitHub-hosted OS labels, record their exact runner
  image/OS identities and every toolchain version in the candidate manifest, and
  pin every build input under repository control. A floating toolchain `stable`,
  container `latest`, unversioned installer, or mutable container base is not
  acceptable; the hosted runner image is recorded rather than falsely claimed to
  be selectable by exact image version.
- Before preparation, require repository settings that make the immutability claim
  real: a `v*` tag ruleset that prevents update/deletion and GitHub immutable releases
  enabled. These are Ian-owned pre-cut settings. The workflow must inspect their
  effective API state before staging and verify the final v0.8.26 release API row is
  immutable; documentation alone is not proof.
- Before the first public write, preflight the PyPI trusted publisher, GHCR package
  permissions, GitHub Release/Pages authority, and `HOMEBREW_TAP_TOKEN`. A missing or
  rejected credential aborts staging; Homebrew may not silently exit zero with a manual
  fallback.
- Build and validate all releasable artifacts before any irreversible public-channel
  write. Use commit-bound GitHub workflow artifacts (not a tag-dependent build) as the
  private staging area; a draft release may be populated only if it does not create or
  require the public tag. Record every staged artifact hash and source SHA in a signed
  or workflow-attested candidate manifest.
- Keep one canonical installer source. docs/install.sh, packaged copies, and the bytes
  served from https://biomcp.org/install.sh must be generated from or byte-identical to
  that source. Installation aborts when no supported SHA-256 tool exists, when the
  checksum sidecar is unavailable or malformed, or when the checksum mismatches. A test
  of a repository-only copy is not deployment proof.
- After all pre-publication gates pass and Ian approves, create the immutable tag at
  the staged SHA as the first public write. Then perform the remaining public writes in a dependency-ordered
  promotion phase without rebuilding: publish immutable PyPI and the versioned GHCR manifest; publish the
  versioned GitHub Release/assets with `make_latest=false`; verify those public assets;
  update and smoke the public Homebrew formula; then move mutable GHCR/GitHub `latest`,
  self-update discovery, and docs pointers and make the final announcement. Homebrew
  cannot be proven while GitHub assets are private. If any public write fails, report a
  partial release loudly and do not mark the release complete; never hide or relabel
  already-published bytes.
- A workflow replay for an older/equal version must not move `latest`, docs, updater
  discovery, or Homebrew backward. Abort if a versioned tag, package, image, release
  asset, or formula version already exists with different bytes; never overwrite an
  already-published versioned artifact.
- Keep v0.8.25's tag, assets, packages, formula, image tags, and checksums immutable.
  Document its provenance mismatch in the v0.8.26 release notes without deleting history.

### 4. Close distribution gaps

#### GitHub native archives

- Release Linux AMD64, Linux ARM64, macOS x86_64, macOS ARM64, and Windows x86_64
  archives plus checksums.
- Before staging and again from the public downloads, prove every executable reports
  `0.8.26` and the same eight-character SHA derived from the tag commit.
- Define the GNU/Linux ABI floor as glibc 2.28 for both x86_64 and ARM64. Build in
  pinned manylinux_2_28 (or byte-equivalent pinned glibc-2.28) environments, inspect
  imported GLIBC symbol versions to reject anything newer, and execute each archive
  in the oldest supported pinned runtime. Keep the generic Linux asset names only
  while this compatibility proof is green; otherwise label the actual floor.

#### PyPI (`biomcp-cli`)

- Publish wheels for the same five supported target combinations, adding Linux ARM64 to
  the current matrix.
- Install every wheel in a clean target-appropriate smoke environment and prove the
  installed `biomcp version` identifies v0.8.26 and the tag commit. The PyPI distribution
  remains `biomcp-cli`; the installed command remains `biomcp`.
- Build Linux wheels in the same pinned glibc-2.28 environments and audit wheel tags
  and bundled libraries before staging; pin maturin and its interpreter/tool inputs.

#### GHCR and Docker

- Publish one `ghcr.io/genomoncology/biomcp:0.8.26` multi-platform manifest with
  `linux/amd64` and `linux/arm64`, then update `latest` only after the versioned image is
  verified.
- Make runtime execution non-root, retain CA certificates, and keep the image usable for
  CLI commands and local stdio MCP. Do not introduce a hosted BioMCP service.
- Give the non-root user an owned, writable home and BioMCP cache/config directories.
  In both architectures run a loopback-fixture-backed CLI lookup and stdio MCP tool
  call that actually writes and rereads cache state; `version`, `list`, and initialize
  alone do not prove a usable runtime.
- Ensure the image consumes the exact pinned release source or the already-verified
  release binary; a separate unverified checkout/build identity is forbidden.
- Emit the standard OCI source/revision/version labels and attach build provenance and
  an SBOM suitable for the Docker MCP Catalog. Use repository-standard signing if one
  exists; do not invent or publish secret material.
- Inspect the pushed manifest and run bounded version/list and stdio-initialize smokes on
  both architectures, using native runners or QEMU as necessary.

#### Homebrew

- Keep the existing `genomoncology/homebrew-biomcp` tap; do not create another tap.
- Render and publish the formula from the verified macOS release checksums.
- The supported Homebrew contract for v0.8.26 is macOS x86_64 and macOS ARM64. Do not
  imply Linuxbrew support unless this ticket deliberately adds and proves Linux formula
  paths. On both macOS architectures, install through the public tap (not the local
  template), run the formula test/version smoke, and prove the installed SHA and version
  match the release commit.

#### MCP Registry metadata and MCPB

- Keep `server.json`, its `biomcp-cli` package entry, `manifest.json`, `CITATION.cff`,
  Cargo metadata/lock, Python metadata/`uv.lock`, and release version in one
  machine-checked version lock.
- Validate the release-stamped `server.json` against the official schema and preserve the
  local stdio package contract for `io.github.genomoncology/biomcp`. The first official
  registry publication remains an Ian-owned post-release action.
- Build exactly `biomcp-0.8.26.mcpb` plus
  `biomcp-0.8.26.mcpb.sha256`. Create one macOS universal executable with `lipo`
  from the two already-verified Darwin binaries and include it alongside the verified
  Windows x86_64 executable. Use MCPB manifest v0.3 OS `platforms` and exact
  `server.mcp_config.platform_overrides.win32.command = "server/biomcp.exe"`
  (plus Windows args when required); do not advertise CPU selection the schema
  cannot express and do not include Linux in this bundle. Validate with the pinned
  official MCPB CLI/schema, inspect the unpacked entry points, and run the bundle on
  native Intel macOS, Apple Silicon, and Windows.
- The bundle must not contain source, local caches, credentials, planning files, or
  workstation paths. Attach the validated MCPB and checksum to the staged/public
  release. A real Claude
  Desktop install smoke and public directory submission remain Ian-owned post-release
  actions because they require his account and desktop client.

### 5. Prove the release from public surfaces

- Make the staging validation job invoke the repository's actual `make lint`,
  `make test`, and `make spec` on the exact release commit before the tag. Do not replace
  them with a hand-maintained approximation. Make `scripts/release-smoke.sh` derive the
  requested version and report path dynamically; remove its hard-coded v0.8.24 name and
  update the tests that currently ratchet that stale value.
- Run `make verify` after the source gates. Investigate and record every provider failure
  or skipped live row. An unavailable provider may be documented only when public
  completeness remains honest and the affected release requirement does not depend on
  it; a schema/contract regression blocks release.
- After publication, download or install from GitHub Releases, PyPI, Homebrew, and GHCR.
  Do not use private build artifacts as post-publication proof.
- Exercise help/version/list, local stdio MCP initialization, the frozen variant identity
  canary, and the frozen ClinGen composition/live diagnostics appropriate to the public
  binary. Preserve provider outages as explicit incomplete/unavailable results.
- Verify `biomcp.org`, `install.sh`, self-update discovery, README install commands,
  checksums, release notes, and all mutable latest pointers identify v0.8.26.
- From an isolated installation of the immediately previous public version, run the real
  self-update download against the new public archive and verify the installed binary's
  version and source SHA. An updater that can discover an asset but cannot download it is
  a release blocker.
- Produce and attach exactly `release-record-v0.8.26.json` and
  `release-record-v0.8.26.json.sha256` after all public smokes. The record contains the
  tag, full source SHA, artifact hashes, wheel targets, OCI manifest
  digests/platforms, SBOM/provenance references, Homebrew formula commit, MCPB hashes,
  all gate results, public smoke results, and explicitly accepted live-provider
  limitations; summarize it in the release notes. If promotion fails partway, attach a
  uniquely named `release-record-v0.8.26-partial-<run-id>.json` instead and never
  overwrite a provisional record under the final name.

### Documentation and mechanical proof

Update these existing authorities as their contracts change:

- `CHANGELOG.md`
- `.github/workflows/release.yml`
- `architecture/technical/overview.md` release pipeline
- `README.md`
- `docs/getting-started/installation.md`
- `docs/getting-started/mcp-clients.md`
- `docs/reference/mcp-server.md`
- `server.json`, `manifest.json`, `Formula/biomcp.rb`, and `Dockerfile`
- the routine release, Docker, Homebrew, MCP, version-sync, and changelog specs/tests

Add routine, frozen failures for the current pre-fix defects: missing Unreleased truth,
event-SHA/tag-SHA divergence, public-before-gates workflow ordering, absent Linux ARM64
wheel, single-architecture GHCR output, root container runtime, missing MCPB payload, and
post-publish checks that inspect only private artifacts. Also freeze committed
tag/version mismatch rejection (including `uv.lock`), old-tag replay protection,
versioned-asset non-overwrite, least-privilege job permissions, immutable action pins,
credential preflight, dynamic release-smoke naming, truthful citation metadata, and the
exact final/partial release-record names. Also freeze tag-ruleset/immutable-release
preflight, exact toolchain/base-image pins, glibc-2.28 symbol/runtime checks, MCPB
universal/Windows entry-point selection, and non-root cache writes. Live publication/provider probes belong under
`make verify`; deterministic workflow/fixture contracts belong under `make spec` or the
corresponding unit suite.

## Out of Scope

- Promoting this draft or publishing v0.8.26 without Ian's explicit go-ahead.
- First publication to the official MCP Registry; Docker MCP Catalog, Claude directory,
  Glama, PulseMCP, mcp.so, Cline, Awesome-MCP, or Smithery submissions/claims.
- A remote hosted BioMCP server, managed hosting, OAuth, or ongoing infrastructure cost.
- Writing or submitting the BioMCP paper, blog post, or demo video. Their Ian-owned
  decisions and follow-through are recorded in the post-v0.8.26 planning checklist.
- Retrospectively replacing or deleting any v0.8.25 public artifact.

## Success Checklist

- [ ] Every frontmatter dependency through 623 is complete, merged, and green; 615
  proves offline composition and 623 proves the unchanged final live release gates.
- [ ] The release notes cover the complete `v0.8.25..release-commit` range, including
  598-609 and 610-616, without claiming fixture-only behavior.
- [ ] The changelog has truthful v0.8.25, v0.8.26, and restored Unreleased boundaries,
  with an offline regression that fails on the pre-fix state.
- [ ] One full tag commit SHA is the source authority for every build, package, image,
  formula, docs deployment, and embedded binary identity.
- [ ] A tag-free candidate run builds and hashes every artifact from the full release
  commit SHA; after approval the tag is the first public write, resolves to that SHA,
  and promotion reuses the staged bytes without a rebuild.
- [ ] Every committed version surface, including `uv.lock`, equals v0.8.26 before the
  tag; CI rejects mismatch and performs no release-time manifest rewrite. Citation
  metadata contains no placeholder DOI or future paper version.
- [ ] A fixture where event SHA differs from tag SHA proves that the tag SHA wins.
- [ ] All release artifacts are built and validated before irreversible public writes;
  the tag is created first at the staged SHA, versioned PyPI/GHCR publish next, the GitHub Release is public with
  `make_latest=false` before Homebrew smoke, and mutable latest/docs/updater pointers
  move last.
- [ ] Immutable action pins, per-job least privilege, credential preflight, old-version
  replay protection, and versioned-artifact non-overwrite all have executable workflow
  contracts.
- [ ] The effective `v*` tag ruleset prevents update/deletion, GitHub immutable releases
  are enabled, and the published v0.8.26 API row reports immutable protection.
- [ ] Rust, targets, uv, maturin, protoc, package tools, and container bases are
  exactly pinned and recorded; fixed hosted-OS labels are used and their exact
  runner image/OS identities are recorded. No controllable release input floats.
- [ ] Linux AMD64/ARM64, macOS x86_64/ARM64, and Windows x86_64 GitHub assets and PyPI
  wheels pass version/SHA and checksum checks.
- [ ] Both GNU/Linux architectures import no GLIBC symbol newer than 2.28 and run in the
  pinned oldest-supported glibc-2.28 environment.
- [ ] GHCR exposes one verified Linux AMD64/ARM64 manifest, runs non-root, carries OCI
  revision/version/source labels, has an owned writable home/cache proven by a cached
  fixture-backed CLI/MCP call, and has attached provenance and an SBOM.
- [ ] Both public Homebrew architecture paths install from the existing tap and identify
  the tag commit; docs state the formula is macOS-only unless Linuxbrew is actually
  proven.
- [ ] `server.json` and all package/citation manifests agree on v0.8.26 and pass schema
  and version-sync checks.
- [ ] `biomcp-0.8.26.mcpb` contains the verified universal macOS and Windows x86_64
  entry points, advertises no unsupported CPU/OS combination, passes native Intel
  macOS/Apple Silicon/Windows smokes, and is attached with its exact checksum.
- [ ] The exact release commit passes `make lint`, `make test`, and `make spec`.
- [ ] `make verify` is run and every non-green result is investigated and included in the
  go/no-go record; no live failure is replaced with a canned success.
- [ ] Public GitHub, PyPI, GHCR, Homebrew, docs, install-script, updater, and MCPB assets
  pass post-publication smoke checks against the same version and source SHA.
- [ ] The public installer fails closed on unavailable/invalid/mismatched checksum proof,
  and the release-smoke report derives v0.8.26 rather than a stale hard-coded version.
- [ ] The public binary passes the frozen variant identity and ClinGen release gates
  without false provider credit or hidden incompleteness.
- [ ] The deployed installer bytes equal the canonical tested installer, and a
  post-deploy smoke proves the public URL aborts when checksum proof is missing,
  malformed, or mismatched.
- [ ] The immediately previous public binary downloads, verifies, installs, and reports
  the new public release through self-update; the smoke uses the published archive rather
  than a private build artifact.
- [ ] v0.8.25 remains byte-for-byte immutable and its provenance defect is documented.
- [ ] Final evidence is attached as `release-record-v0.8.26.json` plus checksum; partial
  attempts use unique partial names and never overwrite the final record.
- [ ] Ian explicitly approves the cut before the tag/publication phase begins.

## Decisions

### 1. One release ticket or separate distribution tickets

**Consideration:** Changelog truth, tag provenance, native artifacts, wheels, containers,
Homebrew, MCP metadata, and MCPB packaging all meet at one release commit. Ian requested
one ticket rather than another chain.

**Options:** Split each channel into a new dependency chain; release current channels and
defer the gaps; keep one release ticket with per-channel mechanical gates.

**Trade-offs:** Splitting reduces individual ticket size but creates more coordination and
can still yield mismatched release states. Deferring gaps knowingly publishes another
incomplete distribution. One ticket is larger, but the publish flow can prove the coupled
release graph as one unit.

**Decision and why:** Keep ticket 499 as the single v0.8.26 release ticket. Each channel
gets a distinct checklist and regression inside the one pinned release boundary.

### 2. Repair v0.8.25 or preserve it

**Consideration:** Public v0.8.25 binaries do not all identify the tag commit. Replacing
assets would make one semantic version refer to different bytes over time.

**Options:** Replace v0.8.25 assets; delete the release; preserve it and correct the
process in v0.8.26.

**Trade-offs:** Replacement hides history and breaks checksum reproducibility. Deletion
breaks users. Preservation leaves a documented imperfect release but keeps history
auditable.

**Decision and why:** Preserve v0.8.25 exactly. Document the mismatch and make v0.8.26
the first fully end-to-end provenance-proven release.

### 3. When the release becomes public

**Consideration:** The current `release: published` trigger exposes GitHub metadata before
distribution succeeds, while public registries cannot provide a true cross-provider
transaction.

**Options:** Keep publishing first and repair failures later; publish every channel in
parallel; privately stage everything, then perform dependency-ordered public writes,
make versioned GitHub assets public before Homebrew, and move only mutable pointers last.

**Trade-offs:** The first two options maximize partial-release risk. Private staging adds
workflow structure; independent public registries can still fail mid-promotion, so a
partial state must remain visible as a failure rather than being falsely called atomic.

**Decision and why:** Build and validate privately from a tag-free release-commit SHA;
after Ian's approval create the tag at that SHA as the first public write and reuse the
staged bytes without rebuilding; publish immutable PyPI and versioned GHCR; publish the versioned GitHub Release with `make_latest=false`; verify public
Homebrew; then move mutable latest/docs/updater pointers and announce. Fail loudly and
stop on any partial publication.

### 4. Linux ARM64 as a supported distribution

**Consideration:** GitHub already ships a Linux ARM64 binary, but PyPI and GHCR do not,
so installation behavior depends on the chosen channel.

**Options:** Remove Linux ARM64; document channel inconsistency; make it first-class in
archives, wheels, and containers.

**Trade-offs:** Removal reduces coverage; documentation preserves a needless trap; full
support adds cross-build and emulated/native smoke work.

**Decision and why:** Linux ARM64 is first-class across GitHub, PyPI, and GHCR, with
architecture-specific public verification.

### 5. Repository automation versus Ian-owned submissions

**Consideration:** The repo can build valid metadata and bundles, but first registry
publication, directory claims, desktop UI installation, and organization submissions
require Ian's accounts or judgment.

**Options:** Pretend documentation proves publication; put credentials into automation;
stop at validated artifacts and hand Ian an exact post-release checklist.

**Trade-offs:** Documentation alone gives false completion. Credential expansion is
unnecessary and risky. The handoff leaves real manual work but keeps authority honest.

**Decision and why:** Ticket 499 owns validated releasable artifacts and public package
channels already supported by repository credentials. Ian owns first-time registry and
directory actions after 499 completes.

### 6. Include the paper in the release ticket

**Consideration:** The paper is important to BioMCP's credibility and visibility, but its
authors, claims, evaluation freeze, venue, preprint, and DOI require human decisions and
work beyond a software release.

**Options:** Block v0.8.26 on the paper; ignore the paper; track it as a named post-release
workstream linked to existing paper plans.

**Trade-offs:** Blocking couples code delivery to publication decisions. Ignoring it loses
momentum. A post-release workstream keeps the benchmark pinned to a real release while
making the human decisions explicit.

**Decision and why:** Do not block v0.8.26 on the paper. Record the paper as a first-class
post-499 checklist, using the released binary and hashes as the reproducible system under
evaluation.

## Dependencies

- Tickets 610-623 are the release code/quality prerequisites and are explicit in the
  frontmatter. Ticket 615 is the final offline ClinGen composition gate and 623 is the
  final live candidate gate; their presence does not make source repairs optional.
- Tickets 598-609 are completed and already form part of the release source history. They
  are release non-regressions, not queue blockers.
- The separate `genomoncology/homebrew-biomcp` tap already exists. Creating it is not a
  dependency or an Ian to-do.
- First-time official MCP Registry and discovery-directory publication begins only after
  this ticket completes; see
  `planning/biomcp/deep-dives/2026-07-23-post-v0.8.26-distribution-and-paper-checklist.md`.

## Notes

- Do not promote this ticket while any dependency is unfinished or until Ian explicitly
  says v0.8.26 is ready to cut.
- Standard gates are `make lint`, `make test`, and `make spec`; there is no `make check`.
  Live-provider proof is `make verify` and follows the frozen source gates.
- The implementation agent receives only this ticket and its worktree. It must inspect
  the current release workflow and current official packaging schemas rather than copying
  stale commands from planning notes.
- Public repo hygiene: never commit workstation paths, planning/notes content, March
  artifacts, tokens, credentials, PHI, private audit data, or raw provider secrets.
- Do not publish a remote hosted BioMCP service. Docker and MCPB are local execution and
  packaging paths only.
- Do not hard-code the illustrative v0.8.25 observed SHAs into release logic. Tests should
  create controlled divergent SHAs and assert the resolved-tag invariant.
