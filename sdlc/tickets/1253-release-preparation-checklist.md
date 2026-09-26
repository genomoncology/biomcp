# Release preparation checklist for 0.9.1

Accumulates the release-prep items recorded across tickets so nothing
hangs on memory. Work this ticket only after Ian says release. Nothing
here runs before that.

## Changelog bullets (from the 1234 dry run)

The coverage gate is right and it lists the missing tickets itself:
run it in dry-run form against the candidate tag and write a
user-facing bullet for every ticket it names, in the Unreleased
section, then flip the heading to `## 0.9.1 — <date>`. Do not trust a
hand-copied list — the 2026-09-26 dry run reported 24 (1220, 1222,
1223, 1224, 1226 through 1237, 1239, 1241, 1242, 1244, 1247, 1248,
1250, 1254), and tickets with bullets already (1240, 1246, 1249,
1251, 1252) drop off as they land.

## Residuals recorded as "exercise at the release run"

- 1221: the leftover CA bundle check on a live host.
- 1222: the release upload itself (container, wheels, tarballs,
  formula) and the Homebrew formula's honesty against the tag.
- 1225: the wheel-stack smoke args after the container legs run.
- 1245/1249: the container build path's first live exercise at the
  tag — watch the pypi-build and build jobs actually run, not skip.

## Ordering

- Tag pushes trigger the release workflow: version bumps and the
  `## Unreleased` flip land on main first, then the tag.
- PyPI upload precedes the Homebrew tap bump; the tap formula's
  version and sha256 come from the published artifacts.
- The docs-live revision gate needs the site's revision file to
  reflect the tag; retry window is 600 s.

## After the tag

- Close #250 (wheel on older Linux) and #282 once the artifacts are
  public and verified.
- The #283 reporter reply (fix shipped, pointer to the patched
  artifact) needs Ian's OK before posting.
- Post-release: sweep the Unreleased residuals from 1241 (Mac
  real-bundle run) and confirm the M5 checklist issue closes.

## Review

- Design review: n/a (checklist)
- Code review: n/a (checklist)
