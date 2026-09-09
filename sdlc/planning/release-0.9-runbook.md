# Publish 0.9 — release and site runbook

Written 2026-08-28, promoted into the repository 2026-09-09 from a working
note. Pre-flight was verified on the machine during 2026-08-27/28; anything
below dated then must be re-checked before the release is cut. Phase 1 is
Ian's by hand — it needs account access the factory does not have.

See `docs/reference/release-process.md` for the mechanics this runbook drives.

## Phase 0 — pre-flight (verified green 2026-08-27/28)

- Full gates: `make test` and `make lint` both exit 0 on current main. Docs
  build strict with zero MkDocs banners.
- Version state: dev identity 0.9.0-dev.6 / 0.9.0.dev6. The changelog was
  truthful through the 1056/1060 era. Tickets 1067-1075 (fulltext `--out`,
  trial-card commands, author markdown, the three lint-hardening contracts,
  the agent-experience surface) were queued or landing.
- Both GitHub issues' fixes verified live in the binary: #249 (indel
  round-trip, 1056) and #248 (server/discover plus stateless 2026-07-28,
  1057/1058).
- Queue otherwise empty. Nothing withdrawn or failed.

None of the queued tickets block starting. Cut the release after them so the
changelog is complete.

## Phase 1 — the release itself (manual, Ian's)

1. Create the single public-version commit: 0.8.25 to 0.9.0 across
   `pyproject.toml`, `Cargo.toml`, `CHANGELOG.md`, `CITATION.cff`, both
   `server.json` manifests, install metadata, and the docs' release-process
   page. All fields must agree. The go/no-go depends on that one commit SHA.
2. Run the `Release candidate` workflow in `stage` mode. It builds and signs
   the 13 artifacts, notarizes, and seals the checksummed manifest. No tag,
   no publish.
3. Review that exact run: signing and notarization evidence, SBOM,
   provenance, live-provider result.
4. Two manual records are required. The MCPB SHA-256 from a Claude Desktop
   smoke test on Windows, and the previous-version updater and installer hash
   trail. Both must name the same source SHA.
5. Decide go, then run `promote` with the source SHA, the stage run ID, and
   both records. Promotion publishes GitHub, PyPI, GHCR and Homebrew, and
   verifies installs from the public locations on all targets before marking
   latest.
6. After promote, close GitHub #248 and #249 with the prepared replies. The
   #248 reply still describes the conformance work as two stages. Both stages
   landed, so tighten that line to say both ship in 0.9.

## Phase 2 — the website and agent readiness

- Confirm that 1073 (`llms.txt` / `llms-full.txt`), 1074 (`.md` twins plus
  the access path) and the robots.txt fix have landed, and that the site is
  redeployed to gh-pages.
- Verify live by hand. `/llms.txt` and `/llms-full.txt` return 200. A docs
  page's `.md` twin returns 200. `Accept: text/markdown` negotiation or the
  chosen query path works. robots.txt carries a real User-agent and Sitemap
  section.
- Re-run the Cloudflare agent-readiness scanner: POST the site URL to
  `isitagentready.com/scan`. It is one-shot per URL, so capture the JSON in
  one pass. The baseline to beat is Level 0 "Not Ready" (2026-08-28:
  malformed robots.txt, llms.txt 404, twins 404, a 91 KB HTML homepage).
- The before-and-after is a blog piece. Hold it until the after-score exists.

## Phase 3 — distribution checks to rerun after publish

- Registry hygiene, from the 2026-08-26 star-uptake investigation. mcp.so
  said "No tools detected" and carried a broken install command. PulseMCP
  carried mirror metadata instead of the official registry name
  `io.github.genomoncology/biomcp`. Smithery had no canonical listing; a
  squatter named `biomcp1` held the name. Update ours where the directory
  allows. The awesome-list descriptions are years stale, describing three
  sources against roughly 30 today; a correction PR is that list's own
  requested mechanism, and opening it is Ian's call under the external-repo
  rule.
- Newsletter submissions with the release: the PulseMCP newsletter, which
  featured BioMCP once already, plus PyCoder's Weekly and Python Weekly. All
  three have open submission forms.
- Snap the metrics before the push and two weeks after: GitHub stars (608 on
  2026-08-26; August was the best month since March), traffic referrers
  (`gh api repos/genomoncology/biomcp/traffic/popular/referrers` —
  chatgpt.com was a top-six source), PyPI download counts, and the MCP
  registry listing.
- The marketing series holds three pieces designed to publish beside the
  release: the trust pair, the MCP-dialects conformance piece, and the author
  pivots. The conformance piece pairs with the #248 close.

## Phase 4 — post-release verification (one pass)

- Install exactly as a user would: `uvx --from biomcp-cli==0.9.0 biomcp
  --version`. Then re-run the three canonical spot checks: the #249 indel
  round-trip, `server/discover` before any handshake with 2026-07-28 in
  `supportedVersions`, and `author papers` plus `article authors`.
- Confirm the published docs site matches the released version string and
  that the changelog page renders the 0.9 entry.
- Record the star and referrer snapshot as the official week-zero baseline.

## Open items parked here (not blocking)

- The robots.txt fix had no ticket as of 2026-08-28. It may also unblock the
  scanner's discoverability check.
- The 1067 `--out` feature and the three lint contracts land before or
  shortly after the release depending on queue timing. Nothing requires
  holding the release for them.
