---
flow: build
priority: 7
deps: ["0957"]
base: edca929f
head: 988844b4
---
# Keep entity-focused responses relevant to the request

Gene and disease responses can offer executable workflow examples containing
unrelated hard-coded entities, and a focused disease section can still fetch
and serialize unrelated treatment and trial enrichment. Keep response guidance
and provider work tied to the entity and sections the caller actually chose.

## Guidance contract

Until a workflow ladder has parameterized commands validated from the current
entity, render only its name, rationale, and stable playbook link. Never emit an
executable command containing example genes, variants, drugs, or diseases from
another entity. Remove API probes whose only purpose is deciding whether to
show those static examples. Existing `_meta.next_commands` that are generated
from the current result remain and must still be tested.

A later ticket may add a parameterized ladder only if every identifier comes
from typed current-result data, its shell arguments use the canonical command
builder, and a process test proves the command parses. An implementation agent
must not invent that expansion in this ticket.

## Focused disease contract

When explicit disease sections are supplied, execute and render only mandatory
identity/provenance plus those sections. In particular, a phenotype-only
request performs no treatment, trial, or other optional enrichment call and
contains none of their fields or next commands. Multiple requested sections
form their union. The existing no-section/default whole-entity response remains
backward compatible.

## Done when

- BRAF and melanoma fixtures contain no ABL1/imatinib/CML or PLN/L39X/
  cardiomyopathy executable examples, while current-entity next commands stay.
- Request-plan/counting fixtures prove static guidance causes no probe and each
  disease section contacts only its named providers; zero network work occurs
  for a validation failure.
- JSON, Markdown, list/catalog content, MCP descriptions, and user guides agree
  on focused versus default behavior.
- A source ratchet rejects known unrelated example literals in runtime workflow
  guidance and rejects hand-built executable ladder strings.

## Authorized test changes

Design commits may restate workflow guidance models/renderers, gene/disease
request planning, disease section serialization, next-command tests, fixtures,
schemas, and docs. Do not add a general workflow engine.

The src line ceiling may rise by at most 180 lines.

## Completion

Runtime workflow metadata now names and explains the relevant installed
playbook without copying unrelated executable examples into entity results or
probing extra providers to decide whether to show guidance. Gene, drug, and
disease paths no longer make those unrelated workflow calls. Explicit disease
sections fetch and render only the requested enrichment while the default card
retains its intentionally broad behavior.
