---
flow: build
priority: 9
deps: ["0928", "0929", "0932"]
---
# Make typed MCP schemas entity-specific

The typed `get` schema currently exposes one union of every entity's section
names and validation accepts a section known to any entity. The typed `search`
tool exposes only generic query and limit fields, hiding useful filters behind
the raw shell-shaped escape hatch.

## Schema contract

Generate discriminated `oneOf` branches from the authoritative command/entity
catalog. Each `get` branch contains one literal entity and only that entity's
identifier, section names, and bounded options. Each `search` branch contains
one literal entity and the filters the shipped typed path actually supports.
An entity/section or entity/filter mismatch fails JSON Schema validation before
dispatch; runtime validation enforces the same rule.

Keep the common typed surface intentionally smaller than the complete CLI.
The following table is the complete typed `search` surface for this ticket;
an implementer must not choose additional fields or omit a listed one:

| Entity | At least one required | Optional fields mapped to CLI |
| --- | --- | --- |
| `author` | `query` | `source=semanticscholar` |
| `gene` | any of `query`, `gene_type`, `chromosome`, `region` | those four fields only; `gene_type` maps to `--type` |
| `pgx` | either or both of `gene`, `drug` | `cpic_level` in `A,B,C,D` |
| `gwas` | either or both of `gene`, `trait` | `p_value` finite and in `(0,1]`; no region field |
| `article` | any of `keyword`, `gene`, `disease`, `drug`, `author` | scalar `gene`; arrays `keyword`, `disease`, `drug`, `author`, `journal`; plus `date_from`, `date_to`, `article_type`, `source`, `open_access`, `no_preprints`, `sort` |
| `trial` | any of `condition`, `intervention`, `mutation`, `criteria`, `biomarker` | those five string arrays plus `phase`, `status`, `source` |
| `variant` | any of `query`, `gene`, `hgvsp` | those fields plus `significance`, `max_frequency`, `consequence`, `review_status`, `revel_min` |
| `protein` | `query` | `all_species`, `reviewed`, `disease`, `existence` |

`article_type` is one of `research-article`, `review`, `case-reports`, or
`meta-analysis`; article `source` is one of `all`, `pubtator`, `europepmc`,
`pubmed`, `semanticscholar`, or `litsense2`; and `sort` is `date`, `citations`,
or `relevance`. Trial `phase` is one of `NA`, `1`, `1/2`, `2`, `3`, or `4`;
`status` is one of `recruiting`, `not_yet_recruiting`,
`enrolling_by_invitation`, `active_not_recruiting`, `completed`, `suspended`,
`terminated`, or `withdrawn`; and `source` is `ctgov` or `nci`. Variant
`review_status` accepts only the canonical strings `0`, `1`, `2`, `3`, `4`,
`none`, `criteria_provided`, and `expert_panel`. `max_frequency` and
`revel_min` are finite values in `[0,1]`. Protein `existence` is an integer
from 1 through 5. Boolean fields are JSON booleans, never truthy strings.

Every scalar string is trimmed, nonempty, and at most 256 Unicode scalar
values. Every listed string-array field contains one to three unique values,
each under that same limit. Dates use the production CLI's accepted `YYYY`, `YYYY-MM`, or
`YYYY-MM-DD` form. Every branch also has `limit` (integer, default 10, range
1-25), `offset` (integer, default 0, range 0-1000), and `json` (boolean,
default false). GWAS additionally enforces ticket 0927's checked
`offset + limit <= 50` window. Schema and runtime validators apply identical
bounds before dispatch.

“At least one” is JSON Schema `anyOf`, not exclusive `oneOf`. Listed filters
may be combined and are all forwarded once; existing CLI/provider validation
then enforces their documented AND semantics. In particular, GWAS `gene` plus
`trait` is valid and required to reach ticket 0928's bounded intersection.
For `trial` with `source=nci`, at most one of `mutation`, `criteria`, and
`biomarker` may be present and its array must contain exactly one value, as
ticket 0929 specifies. There are no
other typed-schema mutual exclusions; contradictory exact variant identities
return the existing typed runtime validation error rather than silently
choosing one.

Typed search deliberately omits `all`, `disease`, `diagnostic`, `phenotype`,
`drug`, `pathway`, and `adverse-event`, plus every CLI filter not named above.
Those valid read-only commands use the documented raw `biomcp` escape hatch;
mutating commands, binary downloads, and local paths remain excluded.

The typed `get` surface is complete and exact: `author` has no entity-specific
field beyond `id`; the other branches are `gene`, `article`, `disease`,
`diagnostic`, `pgx`, `trial`,
`variant`, `drug`, `pathway`, `protein`, and `adverse-event`, with sections
drawn only from the corresponding production `*_SECTION_NAMES` constant.
Each branch has a trimmed `id` of 1-512 Unicode scalar values, `json` boolean defaulting
false, and at most 16 unique sections. `author` has no `sections` property.
Source-specific diagnostic/pathway restrictions remain runtime validation
after the entity-specific schema check; no section from another entity is
accepted.

This is the surface at this ticket's landing. Later ticket 0690 is explicitly
authorized to add only `assembly` to the typed `get variant` branch and to
update the same checked catalog; that planned addition is not schema drift.

## Done when

- Generated schemas cover every intentionally typed search/get entity and no
  section appears under the wrong entity.
- A checked table test proves the published branches, fields, enum values,
  required choices, defaults, and bounds are exactly the contract above; a
  new CLI option does not silently become a typed MCP option.
- Positive examples for article, trial, variant, gene, protein, PGx, GWAS, and
  author reach the expected command exactly once through local fixtures.
- Cross-entity sections, unknown filters, raw byte downloads, and oversized
  values fail before command dispatch.
- Schema size remains inside ticket 0932's aggregate tools/list budget.
- MCP reference examples are generated or checked from the same branches, and
  every example validates against the published schema.

## Authorized test changes

Design commits may restate typed search/get schemas and validation in
`src/mcp/shell.rs`, MCP contract-client fixtures, MCP specs, and generated MCP
reference examples. Existing specialized ClinGen tools, raw allowlist, result
rendering, and safety annotations remain covered.

The src line ceiling may rise by at most 260 lines.
