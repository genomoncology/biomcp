---
name: biomcp
description: Search and retrieve biomedical data - genes, variants, clinical trials, diagnostic tests, articles, drugs, diseases, pathways, proteins, adverse events, pharmacogenomics, and phenotype-disease matching. Use for gene function, variant pathogenicity, trials, diagnostics, drug safety, pathway context, disease workups, and literature evidence.
---

# BioMCP CLI

If you don't know how to start, run `biomcp suggest "<question>"` first, then
open the returned `biomcp skill <slug>` playbook for the full workflow.

## Routing rules

- Start with the narrowest command that matches the question.
- Use `biomcp discover "<free text>"` when you only have a single biomedical phrase and need the CLI to resolve the first typed command. `discover` is a single-entity resolver and single-entity free-text lookup only: use it to resolve the canonical name, ID, or category of one thing. It is not a relational query tool and not a list-question seed. Examples: `biomcp discover BRCA1` and `biomcp discover dabigatran`. Symptom-of-disease prompts, HPO symptom bridges, treatment prompts, gene+disease orientation, and unambiguous gene-plus-topic follow-ups remain supported exceptions.
- Relational or multi-entity questions may redirect to `biomcp search all --keyword "<query>"` instead of surfacing weak collocation matches as if they were a resolved discover answer.
- Use `biomcp search all --gene <gene> --disease "<disease>"` when you know the entities but not the next pivot.
- Treatment questions: `biomcp search drug --indication "<disease>" --limit 5`
- Diagnostic-test questions: use structured pivots first with `biomcp get gene <symbol> diagnostics` or `biomcp get disease "<disease>" diagnostics`; use `biomcp list diagnostic` for the full source/filter/section contract; use `biomcp search diagnostic --gene <symbol> --limit 5`, `biomcp search diagnostic --disease "<disease>" --source all --limit 5`, or `biomcp get diagnostic <id>` when you need the full search/detail surface.
- Symptom or phenotype questions: `biomcp get disease <name_or_id> phenotypes`
- Gene-function questions: `biomcp get gene <symbol>`
- Drug-safety questions: `biomcp drug adverse-events <name>` and `biomcp get drug <name> safety`
- Drug-interaction questions: `biomcp drug interactions <name>` and `biomcp get drug <name> interactions`
- EMA and WHO regional drug data are local runtime files that auto-download on first use, DDInter is the local drug-interaction bundle, CDC CVX/MVX is the companion local vaccine-brand bridge for default/EU vaccine searches plus explicit WHO vaccine search, and GTR plus WHO IVD are local diagnostic-test backbones; run `biomcp ddinter sync`, `biomcp ema sync`, `biomcp who sync`, `biomcp cvx sync`, `biomcp gtr sync`, or `biomcp who-ivd sync` to force-refresh before freshness-sensitive local-runtime lookups.
- Vaccine brand-name questions that miss on MyChem often need `biomcp search drug <brand> --region eu`, omitted `--region`, or explicit `biomcp search drug <brand> --region who --product-type vaccine`, which can bridge through CDC CVX/MVX into EMA or WHO vaccine matches.
- Review-literature questions: `biomcp search article -k "<query>" --type review --limit 5`
- Keyword-only article searches may return `_meta.suggestions[]` objects when the whole keyword exactly matches a gene, drug, or disease label/alias; use the suggested `get gene`, `get drug`, or `get disease` command when structured data may answer before more article paging.
- For repeated article keyword searches in one task, use JSON plus `--session <token>` with a short non-secret local label. If the next keyword overlaps the previous same-session keyword, `_meta.suggestions[]` can point to prior `article batch`, `discover`, or date narrowing instead of more reformulation.
- Some first-call JSON responses include `_meta.workflow` and `_meta.ladder[]`; these are static sidecar-backed multi-step ladders. Treat `_meta.next_commands` as dynamic one-hop follow-ups and `_meta.ladder[]` as the worked-example path for the named workflow.
- After `search article`, default to `biomcp article batch <id1> <id2> ...` instead of repeated `get article` calls. Batch up to 20 shortlisted papers in one call.
- Use `biomcp batch gene <GENE1,GENE2,...>` when you need the same basic card fields, chromosome, or sectioned output for multiple genes.
- For diseases with weak ontology-name coverage, run `biomcp discover "<disease>"` first, then pass a resolved `MESH:...`, `OMIM:...`, `ICD10CM:...`, `MONDO:...`, or `DOID:...` identifier to `biomcp get disease`.
- Multi-hop article follow-up: `biomcp article citations <id> --limit 5` and `biomcp article recommendations <id> --limit 5`

## Section reference

- `get gene ... protein`: UniProt function and localization detail
- `get gene ... hpa`: Human Protein Atlas tissue expression and localization
- `get gene ... expression`: GTEx tissue expression
- `get gene ... diseases`: disease associations
- `get gene ... diagnostics`: GTR diagnostic-test pivot for a gene
- `get article ... annotations`: PubTator normalized entity mentions for standardized extraction
- `get article ... tldr`: Semantic Scholar summary and influence
- `get disease ... genes`: associated genes
- `get disease ... phenotypes`: HPO phenotype annotations; source-backed and sometimes incomplete
- `get disease ... pathways`: pathways from associated genes
- `get disease ... diagnostics`: GTR and WHO IVD diagnostic-test pivot for a condition
- `get diagnostic ... genes`: joined gene names from the GTR detail bundle
- `get diagnostic ... conditions`: joined disease or condition names from GTR
- `get diagnostic ... methods`: source-native GTR testing methods
- `get diagnostic ... regulatory`: opt-in FDA device 510(k) and PMA overlay for supported diagnostic records
- `get drug ... label`: FDA label indications, warnings, and dosage
- `get drug ... regulatory`: regulatory summary
- `get drug ... safety`: safety context and warnings
- `get drug ... interactions`: DDInter-backed structured drug-drug interactions plus source-scoped empty wording
- `get drug ... targets`: ChEMBL and OpenTargets targets
- `get drug ... indications`: OpenTargets indication evidence

## Cross-entity pivot rules

- `gene articles <symbol>` and `search article -g <symbol>` are equivalent starting points for gene-filtered literature.
- Use helpers when the pivot is obvious: `drug interactions`, `drug trials`, `disease trials`, `variant articles`, `article citations`.
- Use sectioned diagnostic pivots for gene or disease contexts: `get gene <symbol> diagnostics` and `get disease <name_or_id> diagnostics`. Use `biomcp list diagnostic` for source/filter/section details, and inspect returned IDs with `get diagnostic <id>` rather than inventing accessions or product codes.
- Use `search article -d "<disease>" --type review --limit 5` when disease phenotypes or drug indications look sparse.
- Use `article batch` as the default multi-article follow-up after `search article`; it replaces sequential `get article` calls and preserves Semantic Scholar enrichment when available.
- Use `batch <entity> <id1,id2,...> --sections <s1,s2,...>` when you need the same card shape for several entities.
- Use `enrich <GENE1,GENE2,...>` once you have a real gene set and want pathways or GO-style categories.

## How-to reference

For question patterns that need more than a one-line routing hint, start with
the executable command or routing phrase below before you improvise the command
sequence.

| Question pattern | Start with | Why |
|---|---|---|
| Specific variant pathogenicity or clinical-evidence question | `biomcp get variant "<variant>"` | Use the bounded variant-pathogenicity workflow instead of mixing ad hoc variant, trial, and article commands |
| Specific drug safety or adverse-event question | `biomcp drug adverse-events <name>` and `biomcp get drug <name> safety` | Start with the drug-safety workflow before widening to literature |
| Drug interaction question for a known medication | `biomcp drug interactions <name>` or `biomcp get drug <name> interactions` | Start with the DDInter-backed helper when the anchor drug is known, then widen to safety or literature only for nuance |
| Drug approval, licensing, or regulatory-date question | `biomcp get drug <name> regulatory` | Use the structured-first workflow discipline: check `get drug ... regulatory` before falling back to articles for approval facts |
| Broad gene-in-disease orientation | `biomcp search all --gene <gene> --disease "<disease>"` | Follow the shipped counts-first workflow for gene, drug, trial, and article pivots |
| Gene-disease association for a known gene | `biomcp get gene <symbol> diseases` | Check `get gene ... diseases` and `search variant --gene ...` for the full disease spectrum before searching articles |
| Gene localization or protein-function question | `biomcp get gene <symbol> protein` and `biomcp get gene <symbol> hpa` | Pull `get gene ... protein` and `get gene ... hpa` first because UniProt and HPA usually answer localization or function directly |
| Diagnostic-test inventory or detail question | `biomcp list diagnostic` and `biomcp get diagnostic GTR000006692.3` | Inspect the shipped diagnostic filters and sections first; use gene/disease diagnostic pivots or `search diagnostic` to get source-native IDs before `get diagnostic` detail |
| You know the concept but not the first entity to inspect | `biomcp search all --keyword "<concept>"` | Use `search all` to choose the next typed command intentionally |
| "Most common" or prevalence question about a disease | `biomcp discover "<disease>"` | Use `biomcp discover` to resolve the canonical disease entity, then inspect structured disease data before widening to article search |
| You already know the anchor entity and want the built-in related view | Use the matching helper such as `biomcp gene articles <symbol>`, `biomcp drug interactions <name>`, `biomcp drug trials <name>`, or `biomcp disease trials "<disease>"` | Move from a known gene, disease, drug, or variant into trials, articles, drugs, pathways, or interaction review without rebuilding the query |
| You need literature for a known gene, disease, drug, method, or outcome | `biomcp search article -k "<query>" --type review --limit 5` | Translate the question into typed flags plus a focused keyword clause |
| You need recruiting or completed trials for a disease, drug, or biomarker | `biomcp search trial -c "<condition>" --limit 5` | Start with condition and intervention filters, then add biomarker or geography only when needed |
| You need to resolve or annotate a variant identifier | `biomcp get variant "<variant>"` | Normalize the variant first, then add significance or frequency filters |
| You need a functional-effect prediction for a variant | `biomcp get variant "<variant>" predict` | Use `predict` only after you have a resolvable variant identifier |
| You need to reproduce a paper-style workflow | Map the paper task to the closest BioMCP entity command, then use `biomcp article batch <pmid1> <pmid2> ...` for shortlisted papers | Map the paper task to the closest BioMCP workflow area before copying commands |
| You need to review whether a workflow run is complete and trustworthy | Check command fidelity, evidence traceability, and reproducibility against the commands already run | Check command fidelity, evidence traceability, and reproducibility before signing off |

## Anti-patterns

### Don't use `discover` for relational or list questions

`discover` resolves one biomedical entity at a time. If the prompt asks for
relationships, classes, or a list, pivot to article keyword search first
instead of treating `discover` as a relational query tool.

- `"drug classes that interact with warfarin"` -> use `biomcp search article -k "drug classes that interact with warfarin" --type review --limit 5`
- `"genes regulated by MEF2 in the heart"` -> use `biomcp search article -k "genes regulated by MEF2 in the heart" --type review --limit 5`, then `biomcp get gene <symbol>` once the literature or question gives you the concrete gene you actually need

### Don't keyword-reformulate

Never do more than 3 article searches for one question. For iterative keyword
searches, pass `--json --session <token>` so BioMCP can detect overlap across
consecutive searches; the token is a local non-secret label, not a user ID. If
two searches with different keywords return similar or empty results, follow
the JSON `_meta.suggestions[]` ladder or change strategy entirely: inspect the
previous result set with `biomcp article batch <id1> <id2> ...`, switch entity
or source, narrow by year, or start with `biomcp discover "<free text>"`.

### Trial nicknames don't work in trial search

ClinicalTrials.gov usually does not index nicknames like `CodeBreaK`, `COSMIC`,
`BEACON`, or `KEYNOTE`. Search by drug plus condition instead, or use
`biomcp search article -k "<trial nickname>"` to recover the NCT ID first.

### Don't use `--type` for niche topics

`--type` reduces recall to Europe PMC publication-type filtering today because
PubTator3 and Semantic Scholar search results do not expose publication-type
filtering. Use it for broad review questions with many results, not sparse or
niche topics.

### Use `--drug` on article search for drug-specific questions

When the question is about a specific drug's trial results, efficacy, or
mechanism, add `--drug <name>` to `search article`. Without the drug filter,
the key results paper often ranks too low to appear on the first page.

### Batch syntax is entity-specific

`biomcp article batch <pmid1> <pmid2> ...` uses spaces between PMIDs. `biomcp
batch gene <gene1,gene2,...>` and `biomcp batch drug <drug1,drug2,...>` use
comma-separated IDs.

## Output and evidence rules

- Quote multi-word IDs or names in commands.
- Do not invent sections, filters, or helper flags that `biomcp list` does not show.
- Treat empty structured regulatory drug results as signal for approved-drug questions, not as a CLI failure.
- Prefer review articles for synthesis questions and structured sections for direct facts.
- Use `_meta.next_commands` from JSON mode as the executable follow-up contract.
- For article search, `_meta.suggestions[]` may contain exact keyword entity matches with `command`, `reason`, and `sections`, or session loop-breaker suggestions with `command` and `reason` only. Multi-concept phrases and typed-filter searches should not produce direct entity suggestions.

## Answer commitment

- Only add more commands if a needed claim is still unsupported. If one command already answers the question, stop searching and answer.
- If a structured section already contains the answer, use it. Anti-pattern: after `biomcp get drug nivolumab regulatory` shows `Sponsor: BRISTOL MYERS SQUIBB`, do not search articles just to confirm who developed nivolumab.
- If 1-2 papers you already fetched state the answer in the abstract or TLDR, answer from those papers instead of hunting for a third paper.
- If 3+ searches keep returning relevant papers, the answer is in what you already have or you need a different approach. If you keep reformulating the same search with different keywords, the answer is in what you already have or you need a different approach. Example: once repeated tau PET or European influenza vaccine searches keep surfacing relevant review papers, stop keyword-churning and extract the answer from those results.

Run `biomcp skill list` for worked examples.
