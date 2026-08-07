---
flow: build
priority: 6
---
# Close the provider-capture directory symlink TOCTOU race

ProviderCaptureStore validates directory components with symlink_metadata then writes by path, so a local attacker can swap a component between the check and the write.

Completed under March on 2026-08-04, as March ticket 647. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/647-close-the-provider-capture-directory-symlink-toctou-race
