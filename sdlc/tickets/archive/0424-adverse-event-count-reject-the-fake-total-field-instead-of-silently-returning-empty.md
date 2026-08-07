---
flow: quickfix
priority: 6
---
# adverse-event --count: reject the fake 'total' field instead of silently returning empty

`biomcp search adverse-event -d <drug> --count total` is accepted but returns empty results — verified: `--count total` yields `"buckets": []` while a real field like `--count patient.reaction.reactionmeddrapt` returns data. openFDA has **no `total` count field**; the report total lives in `meta.results.total`, which is a different concept from a faceted `&count=<field>`.

Completed under March on 2026-06-17, as March ticket 424. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/424-adverse-event-count-reject-the-fake-total-field-instead-of-silently-returning-empty
