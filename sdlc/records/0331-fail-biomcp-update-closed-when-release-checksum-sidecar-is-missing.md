---
base: e7fe487d3c65e470bc614bb30e2e07bde39a3c01
head: 084262d0591de0f14f6491cba32b914cd7f64068
---
`src/cli/update.rs::verify_archive_checksum_if_available()` returns `Ok(false)` when `<asset_url>.sha256` is absent, and `run()` converts that to a warning ("checksum file missing for {asset_name}; continuing without checksum verification"). `replace_current_binary()` then writes and renames the downloaded executable into the current binary path. For a self-update command that downloads and installs an executable, fail-open checksum behavior weakens the supply-chain boundary; the binary is then protected only by TLS plus GitHub release integrity.

Imported from March ticket 331. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/331-fail-biomcp-update-closed-when-release-checksum-sidecar-is-missing
