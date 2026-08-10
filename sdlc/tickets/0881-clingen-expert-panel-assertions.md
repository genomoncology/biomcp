---
flow: build
priority: 6
deps: ["0951"]
---
# Carry the guideline identity on ERepo assertions

This is only the guideline-identity slice. Bounded gene search moved to 0908.

## Done when

biomcp variant erepo <CAID> --detail --json carries:

- guideline_label exactly as ERepo supplied it;
- guideline_version as a parsed semantic version when the label contains one,
  otherwise null;
- doc_version separately, with no suggestion that it is the guideline
  version.

Human output names the same guideline label/version near the assertion
classification. Older labels such as ACMG-PTEN Variant Curation Guideline are
preserved even when no semantic version can be parsed.

## Proof required

The current client uses summary/detail shapes that do not retain guidelines.
Add the provider field/request needed to preserve the association; do not
infer a guideline from VCEP name or document version.

- Pin the exact production RequestPlan.
- Record a dated real response with a versioned guideline and one legacy
  unversioned label.
- Decode through the production parser and assertion model.
- Prove JSON and Markdown, including null parsed version with raw label kept.
- Keep all routine tests local and body-bounded.

Gene-wide enumeration, criterion filtering, CSpec document search, and
unbounded downloads are out of scope.

## Authorized test changes

Design commits may restate ERepo request/parser captures, assertion model
fixtures, CLI JSON/Markdown tests, and schemas/examples that currently omit
the field. Mechanical construction fixes may land with implementation while
unrelated assertions remain unchanged.

The src line ceiling may rise by at most 150 lines.
