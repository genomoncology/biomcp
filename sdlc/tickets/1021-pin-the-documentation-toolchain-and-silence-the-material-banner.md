---
flow: build
priority: 30
---

# Pin the documentation toolchain and silence the Material banner

Decision recorded 2026-08-23, Ian's ruling: BioMCP does **not** migrate
generators. The documentation build stays on MkDocs 1.x with Material for
MkDocs, pinned so the resolver cannot drift, and the deprecation banner is
silenced. The migration options (Zensical, another generator) were considered
and explicitly declined; revisit only if a security advisory or a Python
version bump leaves MkDocs 1.x unusable. Until then the site is stable and
stays exactly as it is.

What is true today, verified on 2026-08-23: `mkdocs build --strict` passes,
the published site is correct, and the red banner is a plain `print()` from
Material's plugin loader (silenceable with `NO_MKDOCS_2_WARNING=1`, the
switch the Material authors provide) — it cannot fail a strict build. The
real risk is the resolver: `pyproject.toml` bounds `mkdocs-material>=9.5`
and does not pin `mkdocs` at all, so a fresh resolve could install MkDocs
2.0, which is incompatible with Material and **would** break the build.

## Done when

- `mkdocs` is a direct dependency of this project, bounded to the 1.x line
  (at least `>=1.6,<2`), and `mkdocs-material` is bounded to stay below 10,
  so no dependency resolution this project performs can install MkDocs 2.x.
- The committed lockfile reflects the bounds, and `uv lock --check --offline`
  passes against them.
- Every documentation build invoked by the repo's gates runs with
  `NO_MKDOCS_2_WARNING=1`, so `make test` and `make spec` no longer print
  the unmaintained-MkDocs banner.
- The strict build still exits 0 and still fails on a broken internal link,
  exactly as it does today.
- Nothing about the site changes: same generator version line, same theme,
  same nav, same URLs. This ticket is a pin and a silenced banner, nothing
  else.

## What this replaces

This ticket replaces its own earlier scope — moving the documentation build
off unmaintained MkDocs — which was withdrawn by the ruling above. A
migration ticket may exist again someday; this is not it.

Filed from `sdlc/issues/2026-08-23-mkdocs-material-compatibility.md`.
