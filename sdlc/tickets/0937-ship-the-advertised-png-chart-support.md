---
flow: build
priority: 8
deps: ["0934"]
---
# Ship the advertised PNG chart support

README, documentation, help, and the chart blog advertise PNG output, but
`charts-png` is outside Cargo's default features. Local builds, native
releases, wheels, and containers all use defaults, so their advertised PNG
command exits with an unavailable-feature error.

## Distribution contract

Make `charts-png` a default BioMCP feature and include it in every public
artifact: local `make build`/`install`, native release archives, PyPI wheels,
containers, and release-candidate smoke artifacts. Source builders may still
disable default features deliberately; ordinary installation documentation
describes the shipped behavior, not that expert opt-out.

## Done when

- From each isolated artifact class, a local study fixture renders a `.png`
  with default scale and with an explicit scale; the file begins with the exact
  eight-byte PNG signature and is nonempty.
- The same artifact still renders terminal and SVG charts.
- A packaging contract inspects Cargo features and every build command so a
  future artifact cannot silently omit PNG while public claims remain.
- README, docs landing page, chart reference/blog, help, and dependency page
  agree that public artifacts include PNG.
- Feature-disabled source-build tests retain the clear unavailable-feature
  error without being mistaken for the public artifact contract.
- The flight must pass the local native artifact smoke and commit the
  push-to-main platform jobs and artifact-smoke contract. Cross-platform
  release-candidate execution is not inferred from workflow text; ticket 0953
  owns the actual Linux/macOS/Windows candidate proof before publication.
- All proof uses local fixtures and no public cBioPortal request.

## Authorized test changes

Design commits may restate Cargo default-feature, Makefile, Docker, release,
wheel, chart fixture, help, and documentation expectations. Existing chart
numeric data, terminal/SVG rendering, output-path safety, and feature-off
behavior remain covered.

The src line ceiling may not rise.
