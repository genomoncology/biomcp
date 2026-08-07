---
base: 2659df6b9da16b2d8f3116fd657b51865bd2f176
head: e4fc721aa7d15d86da1b4210e4a873a73e00f592
---
`make check` flakes on main. The WikiPathways HTML sanitization from ticket 163 swallows 404 errors instead of propagating them as clean error messages. The test `dispatch_section_pathway_surfaces_sanitized_wikipathways_404_without_timeout` asserts `section.error` contains "wikipathways", "HTTP 404", and "HTML error page", but `section.error` is `None` — the sanitization branch returns an empty success (0 results, no error) instead of a sanitized error.

Imported from March ticket 169. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/169-fix-wikipathways-404-sanitization-to-propagate-error
