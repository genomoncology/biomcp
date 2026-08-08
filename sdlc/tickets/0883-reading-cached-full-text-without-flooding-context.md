---
flow: build
priority: 3
---
# Let a caller see inside cached full text without reading all of it

## Done when

`biomcp get article <id> fulltext` states the size and section count
beside the cached path, and an opt-in flag returns the section outline
with line ranges. The default output stays the same size it is today,
whatever the length of the paper.

## The finding

Raised as an issue during BioMCP research on 2026-08-08, then folded in
here and the issue file removed. The text below is the issue as filed.

current behaviour is deliberate and the reason for it is sound.

## The current behaviour and why it exists

`biomcp get article <id> fulltext` prints a path, not the text:

    ## Full Text (NCBI EFetch PMC XML)
    Saved to: /home/ian/.cache/biomcp/downloads/994b4877….txt

This is on purpose. A full paper is tens of kilobytes — the one
above is 54,742 bytes — and dumping it into an agent's context
displaces everything else and pins that cost into the conversation's
cache for the rest of the session. Writing to disk and handing back
a path keeps the transcript small and lets the caller decide.

That is the right instinct. The trade is that the caller now has to
leave BioMCP to use what BioMCP fetched, and has no idea what is in
there before they do. Reading the whole file with a file tool
reintroduces the exact cost the design avoided, and that is what
actually happened in practice.

## What would make the trade a good one

The path should be the *fallback*, not the only option. Some way to
learn what is in the document, and to pull out the part that matters,
without materialising the rest.

Sketches, roughly in order of value per unit of work:

1. **An outline.** Section headings with their byte offsets or line
   ranges. Perhaps 300 bytes for a paper, and it turns "read the
   whole thing to find the methods" into "read lines 210–340". This
   is the one that pays for itself immediately.
2. **A section verb.** `fulltext --section "Population Data"` or
   `--section 4`. With the outline above, this is the natural next
   move and probably the most-used one.
3. **Search within the document.** `fulltext --grep "penetrance"`
   returning matches with a few lines of context. Cheap to
   implement, and it is what a reader with a specific question
   actually wants.
4. **A head.** `--head 2000` for the first N characters. Crude, but
   the abstract and introduction answer a surprising number of
   questions, and it is the lowest-effort thing on this list.
5. **Say how big it is.** Even with none of the above, printing
   `54,742 bytes, 9 sections` alongside the path lets a caller
   decide whether reading it is affordable. Nearly free.

Note that (5) and (1) together already fix most of the pain: the
caller knows the size, knows the shape, and can make an informed
choice about the path.

## A constraint worth stating

Whatever is added should keep the default quiet. The current default
costs about 20 lines of output regardless of paper size, and that
property is the whole point. Every option above should be opt-in, so
the cheap path stays cheap.

## The same question applies elsewhere

`biomcp --json get article <id> assets` returns a manifest that ran
to several kilobytes for a single article with one supplementary
file, most of it repeated provider blocks and discovery routes. Any
convention settled here — a compact default with a verbose flag —
should probably apply there too.

Raised 2026-08-08 from PTEN GN003 research for varclassify2.
