# Discover, Suggest, and Skill

These three commands form BioMCP's onboarding surface: `discover` is primarily
the single-entity resolver for free text plus a small set of already-supported
routed prompts, `suggest` picks a worked-example playbook, and `skill` opens the
longer guide behind that playbook. The canaries here keep that first-move
surface focused on real routing behavior instead of incidental copy.

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

```bash
cargo test --lib ticket_377_discover_renderer_envelope_contracts -- --nocapture \
  | mustmatch like 'ticket_377_discover_renderer_envelope_contracts'
```

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

## Suggest Keeps the Playbook and No-Match Contracts

`suggest` is the offline first move for question routing. Matched responses
should point to the concrete playbook, and no-match should stay successful with
the same four-field JSON shape.

```bash
../../tools/biomcp-ci suggest "What drugs treat melanoma?" | mustmatch like 'matched_skill: `treatment-lookup`
`biomcp skill treatment-lookup`'
../../tools/biomcp-ci --json suggest "What is x?" | mustmatch like '"matched_skill": null'
../../tools/biomcp-ci --json suggest "What is x?" | jq -e '.first_commands == [] and .full_skill == null' >/dev/null
```

## Suggest Decomposition Keeps the First-Move Router Review-Sized

The behavior checks above protect the public playbook response. The router also
needs its documented ownership zones so future route additions do not collapse
back into one large catch-all module.

<!-- mustmatch-lint: skip -->

```bash run id=suggest-structure-contract
cd ../.. && cargo test --test suggest_cli_structure -- --nocapture 2>&1
```

```text expect=suggest-structure-contract contains
suggest_split_files_exist_with_doc_headers
```

## Skill Still Opens the Longer Guide

Once `suggest` points to a playbook, the user still needs both the worked-example
index and the canonical agent guide behind `skill render`. The rendered prompt
should also carry the stricter discover framing and the relational-query
counter-examples so installed `SKILL.md` matches the canonical prompt.

```bash
../../tools/biomcp-ci skill | mustmatch like 'biomcp suggest "<question>"'
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

## Normalize-to-Codes Playbook Uses Live Discover Code Labels

The normalize-to-codes worked example should teach a real `discover` workflow,
not a copied table of canned codes. The playbook opens the command sequence, and
the live JSON response keeps source-labelled ontology and clinical-code labels
visible for downstream structuring agents.

```bash
../../tools/biomcp-ci skill normalize-to-codes | mustmatch like "biomcp discover
MONDO
SNOMED
ICD-10
RxNorm"
```

```bash
../../tools/biomcp-ci --json discover "type 2 diabetes mellitus" | mustmatch like '"primary_id": "MONDO:0005148"
"source": "SNOMEDCT"
"source": "ICD10CM"'
```

## Skill Decomposition Keeps Catalog and Install Ownership Separate

The behavior checks above protect the public skill output. The implementation
also needs separate asset, catalog, and install ownership zones so MCP resource
reads and filesystem installation do not collapse back into one over-cap module.

<!-- mustmatch-lint: skip -->

```bash run id=skill-structure-contract
cd ../.. && cargo test --test skill_cli_structure -- --nocapture 2>&1
```

```text expect=skill-structure-contract contains
skill_split_files_exist_with_doc_headers
```
