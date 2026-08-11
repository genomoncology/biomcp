# Discover and Skill

These routine contracts keep the discover onboarding surface focused on stable local behavior. Captured OLS4 responses are replayed through the public CLI; provider-health checks remain in the live companion page.

## Discover Request Planning Happens Before Source Calls

`discover` normalizes free text into a request-command seam before OLS4,
UMLS, or MedlinePlus clients are constructed. That seam records the trimmed
query, command-versus-alias-fallback mode, OLS4 lookup query, and whether
MedlinePlus/cache behavior is enabled, so routine tests can prove routing intent
without depending on a live ontology service.

## Deterministic Renderer Envelope Contracts

Ticket 377 moves routine discover renderer/envelope proof into fixture-result
contracts. The deterministic tests should cover discover JSON `_meta.next_commands`,
source provenance, discovery source labels, markdown Concepts/Suggested Commands
anchors, and truthful degraded guidance without live OLS4, UMLS, or MedlinePlus
calls.


## Alias-Like Free Text Still Resolves to Typed Follow-Ups

When the query is a familiar alias rather than a canonical gene symbol,
`discover` should still surface the canonical concept and a usable next command.

## Disease-Specific Symptom Phrases Stay Clinically Modest

Queries that ask for symptoms of a known disease should route to disease
phenotypes, keep the resolved disease visible in concepts, and treat
UMLS/MedlinePlus plain-language context as optional enrichment rather than a
baseline requirement.

## HPO-Backed Symptom Phrases Should Bridge into Phenotype Search

The discover guide says symptom concepts with HPO-backed IDs should suggest a
phenotype search first. That keeps symptom-first queries on the phenotype
surface instead of dropping straight into broader disease search.

## Relational Queries Redirect Instead of Surfacing Weak Collocation Noise

`discover` should stay honest about its role: it resolves single entities and a
few routed exceptions, but relational or multi-entity questions should redirect
to `search all --keyword` when only weak residue remains.

### MEF2 relational query

Ticket 371 identified this live OLS4 discover path as a request-contract risk;
routine coverage for the MEF2 relational redirect is now restored through Rust
fixture-backed request-command and request-plan tests. The `DiscoverRequest`
seam records command-mode routing before clients are constructed,
`OlsSearchRequestPlan` asserts OLS4 search construction, and fixture hits prove
the router redirects to `search all --keyword` when only weak general hits
remain. Any live OLS4 upstream probe belongs in a release/live-smoke lane, not
routine `make spec-pr`.

## No-Match Discover Queries Fall Back to Article Search

Free text that does not resolve to a biomedical concept should still end with a
next step rather than a dead end.

## Captured Diabetes Identity

The routine ontology fixture replays the recorded OLS4 diabetes result. The discover surface must expose the stable MONDO identity alongside the resolved label, rather than returning only a free-text suggestion.

```bash
../../tools/biomcp-ci --json discover "type 2 diabetes mellitus" | mustmatch like '"primary_id": "MONDO:0005148"
"label": "type 2 diabetes mellitus"'
```

## Captured No-Match Article Guidance

When the recorded OLS4 no-match response contains no concepts, discover must still give a usable article-search next step instead of ending at an empty result.

```bash
../../tools/biomcp-ci discover "SCENAR therapy" | mustmatch like 'No biomedical entities resolved
biomcp search article -k "SCENAR therapy" --type review --limit 5'
```

## Captured Relational Redirect

The recorded OLS4 response for the MEF2 relational query has no viable single-entity result. Discover must redirect that question to keyword search, preserving the full query instead of surfacing weak collocation noise.

```bash
../../tools/biomcp-ci discover "genes regulated by MEF2 in the heart" | mustmatch like 'biomcp search all --keyword "genes regulated by MEF2 in the heart"'
```

## Observed OLS4 Requests

The fixture accepts only the exact production query shape for the three
captured routes, including the row limit, grouping field, and ontology list.

```bash
grep -F 'GET /ols4/api/search?q=type+2+diabetes+mellitus&rows=10&groupField=iri&ontology=' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | mustmatch like 'mondo%2Cdoid%2Chp'
grep -F 'GET /ols4/api/search?q=SCENAR+therapy&rows=10&groupField=iri&ontology=' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | mustmatch like 'hgnc%2Cmesh%2Cmondo'
grep -F 'GET /ols4/api/search?q=genes+regulated+by+MEF2+in+the+heart&rows=10&groupField=iri&ontology=' "$BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG" | mustmatch like 'wikipathways%2Cso'
```

## Skill Still Opens the Longer Guide

The user needs both the worked-example index and the canonical agent guide
behind `skill render`. The rendered prompt
should also carry the stricter discover framing and the relational-query
counter-examples so installed `SKILL.md` matches the canonical prompt.

```bash
../../tools/biomcp-ci skill | mustmatch like 'biomcp skill list'
../../tools/biomcp-ci skill list | mustmatch like '# BioMCP Worked Examples
treatment-lookup'
../../tools/biomcp-ci skill render | mustmatch like '## Routing rules
## How-to reference
single-entity free-text lookup only
biomcp discover BRCA1
biomcp discover dabigatran
### Don'"'"'t use `discover` for relational or list questions
"drug classes that interact with warfarin"
biomcp search article -k "drug classes that interact with warfarin" --type review --limit 5
"genes regulated by MEF2 in the heart"
biomcp get gene <symbol>'
```

## Skill Decomposition Keeps Catalog and Install Ownership Separate

The behavior checks above protect the public skill output. The implementation
also needs separate asset, catalog, and install ownership zones so MCP resource
reads and filesystem installation do not collapse back into one over-cap module.
