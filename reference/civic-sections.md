# CIViC Sections

BioMCP exposes CIViC as explicit, section-gated enrichment, with one additive exception for drug targets: default `get drug <name>` and `get drug <name> targets` output may include a separate `Variant Targets (CIViC): ...` line when CIViC maps a variant-specific molecular profile to an already displayed generic target.

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
- Drug target output may still include a compact CIViC variant-target annotation line without including the full CIViC evidence table.
- `all` includes CIViC where supported:
  - `biomcp get variant <id> all`
  - `biomcp get gene <symbol> all`
  - `biomcp get drug <name> all`
  - `biomcp get disease <name_or_id> all`
