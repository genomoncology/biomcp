# Trial Search Retirement Surface

Trial search remains a condition, intervention, and detail lookup surface. The
former rare-disease planning controls are absent, so callers use literal
conditions and the existing intervention alias controls instead.

## Retired planner controls are rejected before a source request

<!-- mustmatch-lint: skip -->

The retired flags are not compatibility aliases. The built CLI rejects them as
unknown arguments before selecting a trial source.

```bash run id=retired-action-summary exit=2 stream=stderr
../../tools/biomcp-ci search trial --action-summary --source retired-source
```

```text expect=retired-action-summary contains
unexpected argument '--action-summary'
```

```bash run id=retired-condition-expand exit=2 stream=stderr
../../tools/biomcp-ci search trial --no-condition-expand --source retired-source
```

```text expect=retired-condition-expand contains
unexpected argument '--no-condition-expand'
```

## Discovery pages only teach the retained trial search

Trial help, the command reference, and user guides do not advertise retired
planner flags or condition-match provenance. The retained alias control remains
available for ordinary intervention searches.

```bash
../../tools/biomcp-ci search trial --help | mustmatch not like '--action-summary
--no-condition-expand
matched_condition_label'
```

```bash
../../tools/biomcp-ci list trial | mustmatch not like '--action-summary
--no-condition-expand
matched_condition_label'
```

```bash
cat ../../docs/user-guide/trial.md ../../docs/user-guide/cli-reference.md | mustmatch not like '--action-summary
--no-condition-expand
matched_condition_label'
```

## CTGov condition search sends the supplied condition literally

A CTGov condition search still returns fixture data, but it does not fan out a
Phelan-McDermid condition into a curated deletion-syndrome request. Drug alias
expansion remains separately covered by the trial search examples.

```bash
../../spec/fixtures/ctgov-request-log run ../../tools/biomcp-ci --json search trial -c "Phelan-McDermid Syndrome" --limit 1 \
  | jq -r '.count, (.results[0].nct_id | startswith("NCT"))' \
  | mustmatch like '1
true'
```

```bash
../../spec/fixtures/ctgov-request-log show | mustmatch not like '22q13+deletion+syndrome'
```
