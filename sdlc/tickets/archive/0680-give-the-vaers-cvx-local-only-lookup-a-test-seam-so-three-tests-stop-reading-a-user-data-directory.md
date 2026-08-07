---
flow: build
priority: 10
---
# Give the VAERS CVX local-only lookup a test seam so three tests stop reading a user data directory

Three VAERS tests read a machine-local CVX data directory and block make test for every other ticket.

Completed under March on 2026-08-05, as March ticket 680. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/680-give-the-vaers-cvx-local-only-lookup-a-test-seam-so-three-tests-stop-reading-a-user-data-directory
