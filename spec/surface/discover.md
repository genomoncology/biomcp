# Discover, Suggest, and Skill

These three commands form BioMCP's onboarding surface: `discover` is primarily
the single-entity resolver for free text plus a small set of already-supported
routed prompts, `suggest` picks a worked-example playbook, and `skill` opens the
longer guide behind that playbook. The canaries here keep that first-move
surface focused on real routing behavior instead of incidental copy.

## Alias-Like Free Text Still Resolves to Typed Follow-Ups

When the query is a familiar alias rather than a canonical gene symbol,
`discover` should still surface the canonical concept and a usable next command.

```bash
out="$(../../tools/biomcp-ci discover ERBB1)"
echo "$out" | mustmatch like "# Discover: ERBB1"
echo "$out" | mustmatch like '`biomcp get gene EGFR`'
```

## Disease-Specific Symptom Phrases Stay Clinically Modest

Queries that ask for symptoms of a known disease should route to disease
phenotypes and plain-language context rather than pretending BioMCP can make a
diagnosis from the free text alone.

```bash
out="$(../../tools/biomcp-ci discover "symptoms of Marfan syndrome")"
echo "$out" | mustmatch like "## Plain Language"
echo "$out" | mustmatch like "**Marfan Syndrome** (MedlinePlus)"
echo "$out" | mustmatch like '`biomcp get disease "Marfan syndrome" phenotypes`'
```

## HPO-Backed Symptom Phrases Should Bridge into Phenotype Search

The discover guide says symptom concepts with HPO-backed IDs should suggest a
phenotype search first. That keeps symptom-first queries on the phenotype
surface instead of dropping straight into broader disease search.

```bash
json_out="$(../../tools/biomcp-ci --json discover "developmental delay")"
echo "$json_out" | mustmatch like "HP:0001263"
echo "$json_out" | jq -e '._meta.next_commands[0] == "biomcp search phenotype \"HP:0001263\""' >/dev/null
```

## Relational Queries Redirect Instead of Surfacing Weak Collocation Noise

`discover` should stay honest about its role: it resolves single entities and a
few routed exceptions, but relational or multi-entity questions should redirect
to `search all --keyword` when only weak residue remains.

### Warfarin relational query

```bash
out="$(../../tools/biomcp-ci discover "drug classes that interact with warfarin")"
echo "$out" | mustmatch like "# Discover: drug classes that interact with warfarin"
echo "$out" | mustmatch like '`discover` resolves single entities. For relational questions, try: biomcp search all --keyword "drug classes that interact with warfarin"'
echo "$out" | mustmatch like '`biomcp search all --keyword "drug classes that interact with warfarin"`'
if echo "$out" | grep -Fq "Interact with Friends Less than I Would Because of Hearing Question"; then
  echo "$out"
  exit 1
fi
```

### MEF2 relational query

```bash
out="$(../../tools/biomcp-ci discover "genes regulated by MEF2 in the heart")"
echo "$out" | mustmatch like "# Discover: genes regulated by MEF2 in the heart"
echo "$out" | mustmatch like '`discover` resolves single entities. For relational questions, try: biomcp search all --keyword "genes regulated by MEF2 in the heart"'
echo "$out" | mustmatch like '`biomcp search all --keyword "genes regulated by MEF2 in the heart"`'
if echo "$out" | grep -Eq "RalA downstream regulated genes|Metastatic Carcinoma in the Heart"; then
  echo "$out"
  exit 1
fi
```

## No-Match Discover Queries Fall Back to Article Search

Free text that does not resolve to a biomedical concept should still end with a
next step rather than a dead end.

```bash
out="$(../../tools/biomcp-ci discover zzzxqv)"
echo "$out" | mustmatch like "No biomedical entities resolved."
echo "$out" | mustmatch like '`biomcp search article -k zzzxqv --type review --limit 5`'
```

## Suggest Keeps the Playbook and No-Match Contracts

`suggest` is the offline first move for question routing. Matched responses
should point to the concrete playbook, and no-match should stay successful with
the same four-field JSON shape.

```bash
out="$(../../tools/biomcp-ci suggest "What drugs treat melanoma?")"
echo "$out" | mustmatch like 'matched_skill: `treatment-lookup`'
echo "$out" | mustmatch like '`biomcp skill treatment-lookup`'
json_out="$(../../tools/biomcp-ci --json suggest "What is x?")"
echo "$json_out" | mustmatch like '"matched_skill": null'
echo "$json_out" | jq -e '.first_commands == [] and .full_skill == null' >/dev/null
```

## Skill Still Opens the Longer Guide

Once `suggest` points to a playbook, the user still needs both the worked-example
index and the canonical agent guide behind `skill render`.

```bash
overview="$(../../tools/biomcp-ci skill)"
echo "$overview" | mustmatch like 'biomcp suggest "<question>"'
list="$(../../tools/biomcp-ci skill list)"
echo "$list" | mustmatch like "# BioMCP Worked Examples"
echo "$list" | mustmatch like "treatment-lookup"
render="$(../../tools/biomcp-ci skill render)"
echo "$render" | mustmatch like "## Routing rules"
echo "$render" | mustmatch like "## How-to reference"
```
