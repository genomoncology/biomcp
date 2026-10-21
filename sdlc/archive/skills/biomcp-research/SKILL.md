---
name: biomcp-research
description: Do biomedical literature and variant research with the BioMCP CLI, and file what you learn about the tool itself as issues in the biomcp repo.
user-invocable: true
metadata:
  short-description: Research with BioMCP, and feed back what it lacks
---

# Research with BioMCP

`biomcp` is on PATH; source is `repos/biomcp`. `biomcp list` is the
command reference, `-j/--json` for machine-readable output.

Two jobs run together here. **Answer the question** you were asked,
and **notice what the tool could not do** while answering it. The
second is not a distraction — a research session is the only time
anyone finds out where the gaps are, and if they are not written
down that day they are lost.

## The loop

1. **Web search first, breadth.** Find out what exists — the paper,
   the registry entry, the guideline. BioMCP is precise, not
   exploratory; it answers about things you can already name.
2. **BioMCP for the record.** `get article <pmid>` for the citable
   metadata and abstract, `search article` / `article citations` /
   `article references` to walk outward from a seed.
3. **Go to the primary source.** If the answer lives in a registry,
   a database, or a specification, fetch it directly. Quote it.
4. **Prefer practice over prose.** When a document contradicts
   itself, what people actually did settles it better than what the
   document meant. Say which one you used.

## Context discipline

`get article <id> fulltext` prints a **path**, not the text. That is
deliberate: a paper is tens of kilobytes and dumping it pins that
cost into the session for good. Do not reflexively read the whole
file back.

Read the abstract from `get article <id>` first — it answers more
questions than you expect. If you must open the cached file, grep it
for the term you need, or read a bounded range. Read it whole only
when you are genuinely going to use most of it.

Same for `--json` output. Pipe it through a filter; do not print
manifests to the transcript to look at one field.

## Verify before you trust it

Cross-entity data is joined from several upstreams and the joins can
be wrong. A protein change and a cDNA change on the same line can
come from different transcripts. A `.xlsx` that arrives as
`text/html` is a download placeholder, not a spreadsheet.

Before a fact goes in a deliverable, check it against a second
source or against internal consistency. When something looks off,
query the upstream directly (MyVariant, Europe PMC, NCBI) to find
out whether the defect is BioMCP's or inherited — the answer decides
whether it is a bug or a feature request.

## Open sources BioMCP does not cover

All keyless, all JSON, all worth knowing:

- **ClinGen CSpec registry** — gene-specific classification rules.
  `https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN003`,
  and attachments at `…/cspec/File/id/<entId>/data` using ids found
  inside the payload.
- **ClinGen Evidence Repository** — expert panel assertions with
  their applied evidence codes and guideline version.
  `https://erepo.genome.network/evrepo/api/classifications?gene=PTEN&matchLimit=500`
- **Europe PMC** — open-access checks and full text.
  `https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=EXT_ID:<pmid>&resultType=core&format=json`
- **gnomAD documentation** as source markdown, since the browser is
  a JavaScript app that WebFetch cannot read:
  `github.com/broadinstitute/gnomad-browser/tree/main/browser/help/topics`

## Report honestly

A researched negative is a result. "I looked in these three places
and it is not published" changes what gets built; a silent gap does
not. Say which sources you read, quote the sentence you are relying
on, and label inference as inference — especially when you are
reasoning from what someone did rather than what they wrote.

Paywalls are findings too. Name the article, say it is paywalled,
and say what you could and could not establish without it.

# Feeding back: issues in the biomcp repo

Bugs and feature ideas both go in `repos/biomcp/sdlc/issues/`, one
file each, kebab-case filename that reads as a sentence. Feature
ideas take a `feature-` prefix so the two are separable at a glance.
Open with an `# H1` title and a `Severity:` line
(`blocking` / `should-fix` / `nice-to-have`). The `triage` skill
promotes them to tickets later; that is a conversation with Ian, not
something to do here.

They are opposite kinds of claim and want opposite evidence.

**A bug says the tool is wrong.** Carry the reproduction, the
observed output, the correct value, and the root cause if you found
it. Trace it into the source and cite `file.rs:line`. Say whether
the defect is BioMCP's or inherited from upstream, because that
decides who fixes it. Diagnose before you file — an issue that ends
at "this warning appears" costs the next person the whole
investigation, and you already had the terminal open.

**A feature says the tool is incomplete.** The evidence is different
and it is easy to file the wrong thing.

- **Lead with the question you could not answer**, not the feature
  you want. "Every ACMG frequency criterion is defined on filtering
  allele frequency and nothing here reports it" is an argument.
  "Add FAF support" is a preference.
- **Say what you did instead.** The workaround is the cost estimate.
  Downloading 19MB and grepping it is a strong case for a command;
  one extra web search is not.
- **Check it does not already exist.** Read `biomcp list` and try
  it. Filing a request for a shipped feature wastes everyone's time
  and makes the rest of the batch less trustworthy.
- **Propose a shape, ranked cheapest first**, and be honest that the
  cheapest one may be enough. Most of the value usually sits in the
  first item.
- **Record the negative results you already paid for**, so nobody
  repeats the search. "No expert panel has written this policy
  anywhere; all 122 specifications were searched" is worth as much
  as the request itself.
- **Respect the design that exists.** If behaviour looks wrong but
  was chosen on purpose, say so and argue about the trade rather
  than the choice. A deliberate decision with a bad edge is a design
  issue, not a bug, and framing it that way is what gets it read.

One issue per defect. Cross-reference related ones by filename
rather than merging them — they will be triaged separately.
