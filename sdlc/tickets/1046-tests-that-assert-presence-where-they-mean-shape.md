---
flow: review
priority: 4
---
# Tests that assert presence where they mean shape

Find the places where a test checks that some text appears in rendered output when what the project actually cares about is how that output is arranged.

The known instance: `get article <id> indexing` renders an author, that author's affiliation, and the affiliation's identifiers as three sibling bullets at column zero, so a reader cannot tell people from institutions or count the authors. The test covering that section asserts only that the affiliation text appears somewhere in the string. It passes identically whether the output is correctly nested or completely flat, so the suite stayed green while the output was wrong, and a repair flight sent to fix it found nothing red to reproduce. That instance is being fixed under 1046's sibling ticket 1034 and is not what this review is for — it is the example of the pattern.

## What the worst failure looks like

This project's output is read by people making clinical and genomic judgments, and parsed by tools downstream. A rendering that loses which affiliation belongs to which author, which qualifier belongs to which heading, or which value belongs to which source, is wrong in a way that looks fine — the text is all present, so nothing appears missing. A substring assertion cannot see that class of fault at all, which means the suite reports health it has not established.

The rendering layer is template-driven, and template whitespace control makes the source's apparent structure an unreliable guide to the output. A test is the only place the real shape gets pinned, so a test that declines to pin it leaves the shape unprotected.

## What to look for

Assertions that a value appears in output, where the surrounding feature is about arrangement — nesting, grouping, ordering, association between a parent and its children. Rendered Markdown is the obvious hunting ground; the same pattern is worth checking wherever output is assembled from a template.

Judge by what the assertion would catch, not by how it is written. A presence check is right when presence is the guarantee. It is wrong when the guarantee is structural and presence is standing in for it because it was easier to write.

File what you find as issues. Do not fix them in this flight.
