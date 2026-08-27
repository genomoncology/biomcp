# The population summary's highest-ancestry line ignores denominator size

`biomcp get variant rs1426654 population` reports, in the Genomes summary:

    Genome highest ancestry frequency: 1kg:cdx_XY (1)

Verified against the JSON for the same variant: the genome-side "highest"
rows sit on tiny subgroups — `1kg:cdx_XY` and `hgdp:japanese_XX` at AF
1.0 with AN 16–56 — while the exome side's highest (`eas`, AF 0.994) has
AN 39,686. The markdown summary line prints the raw maximum with no
denominator and no floor, one line above the carefully bounded
`grpmax FAF95` figure. A reader quoting "100% in an ancestry group" from
an n=16 subgroup is being invited into an overclaim the card itself
disclaims elsewhere.

Mechanism, verified in code: `highest_ancestry_frequency`
(`src/render/markdown/variant.rs:145`) filters rows with any AF and takes
the max, with no minimum-AN floor and no AN shown. The JSON carries AN
for every row, so the data supports a honest display.

Fix shape (design settles): show the AN beside the frequency, floor the
"highest" pick on a minimum AN (matching what the FAF line already does
by excluding bottlenecked groups), or label subgroup-prefixed ids
(`1kg:`, `hgdp:`) as small-cohort sources. Also worth fixing while
there: the `.expect()` inside the `max_by` comparator is safe today only
because the filter precedes it — brittle to future edits.

Found in the experiment-193 code hunt, 2026-08-27, against 0.9.0-dev.6.
