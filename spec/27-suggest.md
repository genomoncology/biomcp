# BioMCP Suggest

`biomcp suggest` is the offline first-move router for biomedical questions. It
does not resolve entities or call upstream sources; it chooses one shipped
worked-example playbook, prints two starter commands, and points to the full
`biomcp skill <slug>` workflow.

## Choosing a Treatment Playbook

Treatment questions should route to the treatment lookup playbook. Markdown
output keeps the four contract fields visible and renders starter commands as
copyable shell commands.

```bash
bin="$(git rev-parse --show-toplevel)/target/release/biomcp"
out="$("$bin" suggest "What drugs treat melanoma?")"
echo "$out" | mustmatch like "# BioMCP Suggestion"
echo "$out" | mustmatch like 'matched_skill: `treatment-lookup`'
echo "$out" | mustmatch like 'biomcp search drug --indication melanoma --limit 5'
echo "$out" | mustmatch like 'biomcp search article -d melanoma --type review --limit 5'
echo "$out" | mustmatch like 'biomcp skill treatment-lookup'
```

## JSON Regulatory Routing

JSON mode uses the global `--json` flag and keeps the response to the four
fields used by agents: `matched_skill`, `summary`, `first_commands`, and
`full_skill`.

```bash
bin="$(git rev-parse --show-toplevel)/target/release/biomcp"
json_out="$("$bin" --json suggest "When was imatinib approved?")"
echo "$json_out" | jq -e 'keys == ["first_commands","full_skill","matched_skill","summary"]' > /dev/null
echo "$json_out" | jq -e '.matched_skill == "drug-regulatory"' > /dev/null
echo "$json_out" | jq -e '.first_commands | length == 2' > /dev/null
echo "$json_out" | jq -e '.full_skill == "biomcp skill drug-regulatory"' > /dev/null
echo "$json_out" | mustmatch like '"matched_skill": "drug-regulatory"'
```

## Variant Evidence Routing

Explicit variant identifiers plus clinical-significance wording should outrank
broader disease or treatment wording. The first two commands start with
variant evidence, while the full playbook remains available through `skill`.

```bash
bin="$(git rev-parse --show-toplevel)/target/release/biomcp"
json_out="$("$bin" --json suggest "Is variant rs113488022 pathogenic in melanoma?")"
echo "$json_out" | jq -e '.matched_skill == "variant-pathogenicity"' > /dev/null
echo "$json_out" | jq -e '.first_commands[0] == "biomcp get variant rs113488022 clinvar predictions population"' > /dev/null
echo "$json_out" | jq -e '.first_commands[1] == "biomcp get variant rs113488022 civic cgi"' > /dev/null
echo "$json_out" | mustmatch like '"matched_skill": "variant-pathogenicity"'
```

## Resistance-to-Drug Mechanism Routing

Mechanism questions phrased as `resistance to <drug>` or `resistance against
<drug>` should still route to the `mechanism-pathway` playbook. The starter
commands must anchor on the drug, not on filler text from the surrounding
question.

This keeps natural-language resistance questions on the same two-command
mechanism workflow that already serves direct `<drug> resistance` phrasing.

```bash
bin="$(git rev-parse --show-toplevel)/target/release/biomcp"
out="$("$bin" suggest "What is the mechanism of resistance to imatinib?")"
echo "$out" | mustmatch like 'matched_skill: `mechanism-pathway`'
echo "$out" | mustmatch like 'biomcp search drug imatinib --limit 5'
echo "$out" | mustmatch like 'biomcp get drug imatinib targets regulatory'
```

## Shell Quoting

User-derived anchors are rendered as command strings, not executed by
`suggest`. Multiword or shell-significant anchors should be quoted in every
starter command so agents can copy the command safely.

```bash
bin="$(git rev-parse --show-toplevel)/target/release/biomcp"
out="$("$bin" suggest "What drugs treat lung cancer; rm -rf /?")"
echo "$out" | mustmatch like 'biomcp search drug --indication "lung cancer; rm -rf /" --limit 5'
echo "$out" | mustmatch like 'biomcp search article -d "lung cancer; rm -rf /" --type review --limit 5'

resistance_out="$("$bin" suggest "What is the mechanism of resistance to imatinib; rm -rf /?")"
echo "$resistance_out" | mustmatch like 'biomcp search drug "imatinib; rm -rf /" --limit 5'
echo "$resistance_out" | mustmatch like 'biomcp get drug "imatinib; rm -rf /" targets regulatory'
```

## More Shipped Question Shapes

The router covers the shipped worked-example catalog with conservative phrase
matching. These examples prove additional high-value routes without requiring
network fixtures.

```bash
bin="$(git rev-parse --show-toplevel)/target/release/biomcp"
"$bin" --json suggest "Where is OPA1 localized?" | jq -e '.matched_skill == "gene-function-localization"' > /dev/null
"$bin" --json suggest "Are there recruiting trials for melanoma?" | jq -e '.matched_skill == "trial-recruitment"' > /dev/null
"$bin" --json suggest "How do I distinguish Goldberg-Shprintzen syndrome vs Shprintzen-Goldberg syndrome?" | jq -e '.matched_skill == "syndrome-disambiguation"' > /dev/null
"$bin" --json suggest "Is Borna disease virus linked to brain tumor?" | jq -e '.matched_skill == "negative-evidence"' > /dev/null
"$bin" --json suggest "Where is OPA1 localized?" | mustmatch like '"gene-function-localization"'
```

## No Match Stays Successful

Low-confidence input should not throw a runtime error. JSON no-match output
uses the same four fields, with null values and an empty command list.

```bash
bin="$(git rev-parse --show-toplevel)/target/release/biomcp"
json_out="$("$bin" --json suggest "What is x?")"
echo "$json_out" | jq -e '.matched_skill == null' > /dev/null
echo "$json_out" | jq -e '.first_commands == []' > /dev/null
echo "$json_out" | jq -e '.full_skill == null' > /dev/null
echo "$json_out" | mustmatch like '"matched_skill": null'
```

## Command Discovery Includes Suggest

The static command reference must teach `suggest` as a first move and provide a
focused `biomcp list suggest` page for the response fields and no-match
behavior.

```bash
bin="$(git rev-parse --show-toplevel)/target/release/biomcp"
out="$("$bin" list)"
echo "$out" | mustmatch like 'suggest "What drugs treat melanoma?"'
echo "$out" | mustmatch like '- `suggest <question>`'

detail="$("$bin" list suggest)"
echo "$detail" | mustmatch like '`suggest <question>` - route a biomedical question'
echo "$detail" | mustmatch like "matched_skill"
echo "$detail" | mustmatch like "first_commands"
echo "$detail" | mustmatch like "No confident BioMCP skill match"
```
