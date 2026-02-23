# First Query

This walkthrough shows the shortest useful BioMCP session.

## 1. Confirm API connectivity

```bash
biomcp health --apis-only
```

You should see an API status table and a summary line.

## 2. Search for a target

```bash
biomcp search gene -q BRAF --limit 5
```

Use search to identify canonical identifiers before deeper retrieval.

## 3. Retrieve a specific record

```bash
biomcp get gene BRAF
```

## 4. Expand with a section

```bash
biomcp get gene BRAF pathways
```

This positional section model is the default way to request deeper detail.

## 5. Try JSON output

```bash
biomcp --json get gene BRAF
```

## 6. Cross to another entity

```bash
biomcp search trial -c melanoma --status recruiting --limit 3
biomcp get trial NCT02576665 eligibility
```

## What to do next

- Read [progressive disclosure](../concepts/progressive-disclosure.md).
- Open an entity guide in `docs/user-guide/`.
- Configure optional keys in [API keys](api-keys.md).
