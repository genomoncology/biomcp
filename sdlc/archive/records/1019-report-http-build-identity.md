---
base: 5efd6057fd87a4d06ddbcdadeff9a3d99181a227
head: 20d02b805a67d0cecc7f05cc48ef96f8fded3a7e
---

# Report HTTP build identity

The HTTP status route exposed only the package version while the CLI reported
the complete compiled build identity.

The repair uses the shared build identity for `GET /`, returning `version`,
`git_revision`, and `build_timestamp`. It adds an HTTP comparison test and
documents the additive fields.
