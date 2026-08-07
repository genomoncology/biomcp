---
flow: spike
priority: 5
---
# Profile command-level article spec runtime before choosing another optimization

Ticket 502 reduced the routine article fixture from 15 server launches to one but merged without its required same-binary before/after measurement. A post-merge review measured current warm `make spec` at 739.08 seconds with a 0.30-second Cargo freshness check. The ticket-501 timing record reports the pre-502 full gate at 704.28 seconds including 114 seconds of compilation, so fixture startup was not demonstrated to be the dominant bottleneck.

Completed under March on 2026-07-13, as March ticket 505. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/505-profile-command-level-article-spec-runtime-before-choosing-another-optimization
