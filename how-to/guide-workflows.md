# How to: follow guide workflows

BioMCP ships three deterministic investigation workflows in its embedded agent
guide. This page publishes those workflows directly in the docs so you can
follow them from the CLI without installing the guide into an agent directory.

The command lines and workflow guardrails here mirror the embedded
`skills/SKILL.md`. For the validation rubric that explains when a workflow run
is complete and trustworthy, see [Skill Validation](skill-validation.md).

## Variant Pathogenicity Workflow

**When to use:** you have a specific variant and need a focused answer about
pathogenicity and clinical evidence.

```bash
biomcp get variant "<id>" clinvar predictions population
biomcp get variant "<id>" civic cgi
biomcp variant trials "<id>"
biomcp variant articles "<id>"
```

Only add more commands if a needed claim is still unsupported.

Example:

```bash
biomcp get variant "BRAF V600E" clinvar predictions population
biomcp get variant "BRAF V600E" civic cgi
biomcp variant trials "BRAF V600E"
biomcp variant articles "BRAF V600E"
```

## Drug Safety Workflow

**When to use:** you need a concise safety summary for a specific drug.

Quick safety summary:

```bash
biomcp get drug <name> label interactions approvals
biomcp drug adverse-events <name>
```

Filtered FDA adverse-event check:

```bash
biomcp get drug <name> label interactions approvals
biomcp search adverse-event --drug <name> --outcome death --limit 10
```

Do not write `biomcp drug adverse-events <name> --outcome ...`.

Vaccine-focused aggregate safety summary:

```bash
biomcp search adverse-event "<vaccine name>" --source all --limit 10
biomcp search adverse-event "<vaccine name>" --source vaers --limit 10
```

Use `--source vaers` when you specifically want the aggregate CDC WONDER VAERS
summary; leave the default `--source all` path when you want FAERS plus VAERS
for vaccine queries.

Example:

```bash
biomcp get drug pembrolizumab label interactions approvals
biomcp drug adverse-events pembrolizumab
biomcp search adverse-event "COVID-19 vaccine" --source all --limit 10
```

## Broad Gene-Disease Workflow

**When to use:** you want broad but bounded context on a gene in a specific
disease.

```bash
biomcp search all --gene <gene> --disease "<disease>" --counts-only
biomcp get gene <gene> pathways diseases protein druggability civic
biomcp search drug --target <gene> --indication "<disease>" --limit 10
biomcp search trial -c "<disease>" --mutation "<gene>" --status recruiting --limit 10
biomcp search article -g <gene> -d "<disease>" --sort citations --limit 10
```

Rules:

- do not run `search disease` unless you need an ontology ID or phenotype sections
- do not use free-text `search drug` when `--target` or `--indication` is enough
- do not run both `search drug <gene>` and `search drug --target <gene>` in the same investigation
- `get variant` only for simple substitutions or exact IDs copied from search results
- do not `get variant` on exon-level free text like `"Exon 19 Deletion"`
- for EGFR/NSCLC, cover exon 19 deletions and exon 20 insertions from disease, drug, trial, or article evidence unless an exact variant ID is surfaced
- if you need a variant deep dive, choose at most two exemplar simple substitutions such as `L858R` and `T790M`
- choose at most two exemplar variants for deep follow-up
- choose at most three representative EGFR drugs for deep follow-up; do not fetch near-duplicates like both `erlotinib` and `erlotinib hydrochloride` unless the distinction matters
- fetch only one or two key articles or trials unless the prompt explicitly asks for exhaustive evidence
- stop once you can cover: gene role/pathway, actionable alterations, approved drugs, active trials, and resistance mechanisms

Example:

```bash
biomcp search all --gene EGFR --disease "non-small cell lung cancer" --counts-only
biomcp get gene EGFR pathways diseases protein druggability civic
biomcp search drug --target EGFR --indication "non-small cell lung cancer" --limit 10
biomcp search trial -c "non-small cell lung cancer" --mutation "EGFR" --status recruiting --limit 10
biomcp search article -g EGFR -d "non-small cell lung cancer" --sort citations --limit 10
```

## Evidence Discipline

Across all workflows:

- only claim facts the current command output supports
- keep the IDs that make the evidence rerunnable, such as variant IDs, PMIDs, and NCT IDs
- prefer source-tied phrasing such as `ClinVar shows...`, `CIViC reports...`, or `the retrieved trials include...`
- if you need a mechanism, approval, or article detail, fetch the section or article that shows it
- if an exact count or score is not clearly visible in the output, summarize qualitatively instead of guessing
- avoid words like `definitive` or `proves` unless the retrieved evidence justifies that certainty
- if one command already answers the question, stop searching

## See Also

- [Skills](../getting-started/skills.md)
- [Skill Validation](skill-validation.md)
