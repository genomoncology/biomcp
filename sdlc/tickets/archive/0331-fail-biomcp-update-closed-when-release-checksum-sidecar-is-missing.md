---
flow: build
priority: 8
---
# Fail biomcp update closed when release checksum sidecar is missing

`src/cli/update.rs::verify_archive_checksum_if_available()` returns `Ok(false)` when `<asset_url>.sha256` is absent, and `run()` converts that to a warning ("checksum file missing for {asset_name}; continuing without checksum verification"). `replace_current_binary()` then writes and renames the downloaded executable into the current binary path. For a self-update command that downloads and installs an executable, fail-open checksum behavior weakens the supply-chain boundary; the binary is then protected only by TLS plus GitHub release integrity.

Completed under March on 2026-04-28, as March ticket 331. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/331-fail-biomcp-update-closed-when-release-checksum-sidecar-is-missing
