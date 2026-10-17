# 2146 build record — refuse an empty supplement download

## Report being fixed

PMID 30209399 (PMC6181777): `get article 30209399 asset
europe-pmc-a88699323445e1fab52547c22f2b5903b5d7429286dd006f08e8882cb458e6f9--NIHMS1501643-supplement-2.xlsx
--output empty.xlsx` on 0.9.2 exits 0 and writes a zero-byte file, while the
PMC OA key for the same filename returns the real 2,306,341-byte file (MD5
c17f1a475560db5b98d83f9e14e8465f).

## Root cause — the Europe PMC route's empty reply is real

Direct capture of Europe PMC's supplementary archive (recorded 2026-10-16,
read-only public GET):

```
GET https://www.ebi.ac.uk/europepmc/webservices/rest/PMC6181777/supplementaryFiles
→ 200, content-type application/zip, 176-byte ZIP whose single member is
  NIHMS1501643-supplement-2.xlsx with declared file size 0
  (compress_size 2 = an empty deflate stream; hexdump confirmed
  local and central headers both declare 0)
```

Not a wrong URL and not an unfollowed redirect: Europe PMC's own archive
stores this supplement as a genuinely zero-length member. The bytes exist
only in the PMC OA package. So there was no Europe PMC URL to fix; the
defect is that BioMCP delivered that zero-byte member as a successful
download.

## Change

- `src/entities/article/mod.rs`: new `ArticleAssetNamedOutcome::EmptyResponse`
  (JSON `empty_response`).
- `src/entities/article/assets.rs`:
  - Every download route that answers with zero bytes — Europe PMC
    supplementary ZIP members, PMC OA package members, Figshare downloads,
    and linked-file fetches — is marked in `coverage` with the
    `empty_response` outcome instead of being listed in `assets`, so
    `get article <id> asset <key>` exits 1 with
    `article_asset_not_retrievable: '<file>' cannot be retrieved
    (empty_response)` and `--output` writes no file.
  - A zero-byte route keeps the asset-key form it would have carried as a
    listed asset and keeps counting toward collision keys, so when one
    filename has several routes the route that returned bytes keeps its
    exact key, the empty route keeps its exact key (the ticket's reported
    `europe-pmc-a8869...--NIHMS1501643-supplement-2.xlsx` key still resolves
    — to the refusal), and a bare filename that is now unique downloads the
    full copy.
  - `named_outcome_priority` puts `EmptyResponse` above `SourceUnavailable`
  (a real reply explains the route better than a transport failure);
  `build_assets_manifest` and `build_europe_pmc_manifest` (the not-included
  surface) no longer count zero-byte members as assets; the asset-manifest
  cache schema moves 1 → 2 so entries cached by 0.9.2 that still advertise
  empty assets are discarded.
- `src/sources/europepmc.rs`: unchanged. The parser intentionally keeps
  zero-length members; a pinned unit test documents that the classification
  lives in asset resolution, not in the ZIP reader.
- Tests: two recorded-reply terminal tests against the article fixture
  (22663024 reproduces the two-route shape — full copy only in the PMC OA
  package, Europe PMC member zero-length; 22663025 has only the empty
  route), three unit tests (collision-key stability, lone-route filename
  key, zero-byte ZIP member passthrough), and a reader-facing section in
  `spec/entity/article.md`.
- Docs: `docs/user-guide/supplementary-materials.md` gains the
  `empty_response` row and the refusal contract; `docs/user-guide/article.md`
  states the rule. `CHANGELOG.md` carries the Fixes bullet.

## Red on main

Job `1792189610534-1839260` (Linux check host) ran the new tests at `cdeddfce7` =
origin/main `48366958c` + tests/fixture only, no product change:

- `empty_supplement_reply_fails_the_download_and_writes_no_file` FAILED —
  on main the command exits 0 with empty stderr and writes the zero-byte
  file.
- `two_route_supplement_prefers_bytes_and_marks_the_empty_alternative`
  FAILED — on main both routes list in `assets` (left 2, right 1).

## Green evidence

| Job | Machine | Command | Verdict |
| --- | --- | --- | --- |
| 1792190865208-1915650 | Linux check host | targeted nextest: the five new tests | passed, 5/5 |
| 1792191220555-1920355 | Linux check host | `make test` at `aeb31fbb9` | passed |
| 1792191787467-1963604 | Linux check host | `make lint` at `71aff3567` | passed |
| 1792191793549-1964706 | Linux check host | `make spec` at `71aff3567` | passed |
| CI run 38979280962 | GitHub Linux | full ci.yml at `71aff3567` | green (all executed lanes) |
| 1792192449179-24743 | macOS build host | live probe at `71aff3567` | see below |

### Live probe (read-only public GETs, macOS build host, job 1792192449179-24743)

- The ticket's exact reported command now exits 1, writes no file, and says:
  `article_asset_not_retrievable: 'NIHMS1501643-supplement-2.xlsx' cannot be
  retrieved (empty_response). Asset key: europe-pmc-a8869932...`
- `asset NIHMS1501643-supplement-2.xlsx` (bare filename, now unique) exits 0
  and downloads the full copy, MD5 c17f1a475560db5b98d83f9e14e8465f — the
  ticket's evidence MD5.

## Findings

- `make lint` cannot run on the macOS build host yet: no Darwin-arm64 archive for the
  pinned ShellCheck 0.10.0 / actionlint 1.7.12. The lint gate for this
  ticket ran on the Linux check host (the machine that has the pinned tools) and in CI.
  Whoever owns the macOS build host gate-host setup should install those two tools.
- `cargo nextest` emits a pre-existing `unused import: std::fs` warning for
  `src/cache/manager.rs:746` (present on main since `d59599c88`, shadowed
  import in the tests module). Not touched by this ticket; clippy's lint
  lane passes on main, so the warning appears only in the nextest build.
- Fixture PMIDs 22663024/22663025 mirror the observed PMC6181777 archive
  shape with scrubbed bytes, keeping the copyrighted XLSX out of the repo.
