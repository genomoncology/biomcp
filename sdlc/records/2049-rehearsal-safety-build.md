# 2049 build record — the release rehearsal is safe and green

Landed 2026-10-11 through merge e0c8d1030. The production-only publish
steps carry repository guards (PyPI upload, Homebrew push), every image
reference derives from the repository running the workflow, the docs
publication gates on the production repo, the Dockerfile's source label
follows the repository, and the changelog gate counts a bullet's closing
ticket group wherever punctuation trails it. The green rehearsal is run
38143192187 in genomoncology/biomcp-release-rehearsal on the candidate
e0c8d1030: every job succeeded through publish-release, every smoke
printed exactly biomcp 0.9.2, the wheels passed the credential-free
check, and PyPI still serves 0.9.1. Reviews: the lane head reviewed
twice (two blockers found and fixed at 4b0642209, then ACCEPT).
