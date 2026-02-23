# CIViC Sections

BioMCP exposes CIViC as explicit, section-gated enrichment.

## Variant

```bash
biomcp get variant "BRAF V600E" civic
```

Returns cached CIViC evidence from MyVariant plus GraphQL-enriched evidence and assertions when available.

## Gene

```bash
biomcp get gene BRAF civic
```

Returns CIViC evidence/assertion totals and representative rows for the gene query.

## Drug

```bash
biomcp get drug vemurafenib civic
```

Returns CIViC therapy-context evidence and assertions.

## Disease

```bash
biomcp get disease melanoma civic
```

Returns CIViC disease-context evidence and assertions.

## Notes

- CIViC sections are opt-in and are not included in compact default output.
- `all` includes CIViC where supported:
  - `biomcp get variant <id> all`
  - `biomcp get gene <symbol> all`
  - `biomcp get drug <name> all`
  - `biomcp get disease <name_or_id> all`
