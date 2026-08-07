---
base: 2c127de4bce302ca525f234d8bc0acde986a60ec
head: efddeb22c809b4e9f96f825bf3aa49bda68dcf0c
---
`src/cli/list.rs` is 1,534 lines and currently mixes the top-level router with 23 hard-coded page builders and an inline test block. The command works, but its static reference pages have no ownership boundaries, so even a one-page edit requires navigating a giant flat file and risks re-growing the CLI reference surface past the architecture cap.

Imported from March ticket 323. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/323-decompose-list-rs-into-src-cli-list-grouped-surface-modules
