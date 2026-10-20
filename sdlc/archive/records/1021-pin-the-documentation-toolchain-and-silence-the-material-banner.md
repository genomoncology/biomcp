---
base: 64c3626468e5b8ae912412f8867cd5f05ed27293
head: b9b7a88e7f90fdabcea94f944e76a4faec6f4b78
---

# Pin the documentation toolchain and silence the Material banner

The development dependency set now directly bounds MkDocs to 1.x and Material
to 9.x, with matching offline lock metadata. Repository-owned strict
documentation builds set Material's supported warning suppression while
preserving strict broken-link failures and the existing site generator, theme,
navigation, and URLs.
