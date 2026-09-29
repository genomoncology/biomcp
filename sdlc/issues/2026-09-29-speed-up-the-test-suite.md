# Speed up the test suite

Ian approved this move to 1.0 on 2026-09-29 (recorded in the review file 2026-09-29-review-of-the-enrichr-cache-and-records-round.md and main commit 5bb12367). A gate should fit one ticket's feedback loop. Today the full suite takes roughly twenty minutes on the gate host and CI takes about thirty-five; splitting slow lanes (the release-profile contracts, the fixture lifecycles, the gencc store tests) into independent jobs shrinks the wall time per merge.

Owner: the developer agent on Ian's queue. Trigger: the 1.0 feature-track start, or any merge whose CI exceeds thirty minutes.
