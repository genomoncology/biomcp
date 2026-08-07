---
flow: quickfix
priority: 9
---
# Fix WikiPathways 404 sanitization to propagate error

`make check` flakes on main. The WikiPathways HTML sanitization from ticket 163 swallows 404 errors instead of propagating them as clean error messages. The test `dispatch_section_pathway_surfaces_sanitized_wikipathways_404_without_timeout` asserts `section.error` contains "wikipathways", "HTTP 404", and "HTML error page", but `section.error` is `None` — the sanitization branch returns an empty success (0 results, no error) instead of a sanitized error.

Completed under March on 2026-04-10, as March ticket 169. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/169-fix-wikipathways-404-sanitization-to-propagate-error
