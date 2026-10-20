# Collect the existing release-metadata checks

One local command, scripts/check-release-metadata.py, runs the existing Python and Rust metadata checks, both existing version-checker modes, candidate changelog coverage and the actual runbook audit. It accepts candidate SHA, proposed version and tag, runs every group after a failure and reports every failed group. The release checklist and runbook name it. No new validation rules or framework were added.

Developer source 6c34619ae48d3b89a00a0874ff1136910ba2b1e1 passed M5 job 2290/1792506824443-44352: 151 Python checks, eight Rust checks, versions/tag, 88-ticket coverage, actual runbook audit, Ruff, tracked-text, quality ratchet and whitespace. Rust selection is restricted to the library and existing package_build_identity and cli_global_modifiers targets; the same eight controls run without compiling unrelated integration binaries.

Historical job 2290/1792506690970-99731 recreated exact original metadata commit fc1e2deb62dc7e122aad7677d7f4c703d7f3f21c from parent b7d6f83acc14eab10b832eecb133ca9d3c9a4672, tree ed7a1c820dc22f101e0b018151e89bf23beb8957. A single invocation reported both the actual runbook mismatch and original development-cycle changelog test failure, then ran all eight Rust controls successfully. This was a successful reproduction of two failed groups, not a claim the old metadata passed.

After the developer stopped, the coordinator merged accepted main 6e664cfa. No metadata-command product input changed in that integration. Raw runs remain under experiment 2102-biomcp-2258-final-live-replay/metadata-command/2290. Source review and normal landing/main gates follow; no full rehearsal was run by this lane.
