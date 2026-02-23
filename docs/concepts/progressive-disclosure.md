# Progressive Disclosure

Progressive disclosure is the primary interaction model in BioMCP.

The model is simple:

1. search broadly,
2. get a specific record,
3. request only the section you need.

## Why this pattern exists

Biomedical APIs can return large nested payloads.
Sending full payloads by default wastes time and context space.

Progressive disclosure keeps default output concise and predictable.

## Three-step workflow

### Step 1: Search

Use `search` to identify candidates.

```bash
biomcp search article -g BRAF -d melanoma --limit 5
```

### Step 2: Get

Use `get` to anchor on a specific identifier.

```bash
biomcp get article 22663011
```

### Step 3: Expand by section

Request a specific section only when needed.

```bash
biomcp get article 22663011 fulltext
```

## Entity examples

Gene:

```bash
biomcp get gene BRAF
biomcp get gene BRAF pathways
```

Variant:

```bash
biomcp get variant "BRAF V600E"
biomcp get variant "BRAF V600E" predict
```

Trial:

```bash
biomcp get trial NCT02576665
biomcp get trial NCT02576665 eligibility
```

Drug:

```bash
biomcp get drug carboplatin
biomcp get drug carboplatin shortage
```

## Operational benefits

- Lower token usage for LLM-based workflows.
- Less irrelevant output in terminal usage.
- Faster debugging because each request has a narrow scope.
- Easier automation because command intent is explicit.

## Practical guidance

- Start with `search` when you are unsure of identifiers.
- Use one section at a time before requesting multiple sections.
- Keep `--json` for downstream systems that require structured parsing.

## Relationship to MCP resources

MCP resources in BioMCP encode this same pattern in reusable workflows.

Use resources when you need guidance on sequence,
and tools when you are ready to execute commands.

## Summary

Progressive disclosure is not a cosmetic choice.
It is the mechanism that keeps BioMCP usable for both humans and agents at scale.
