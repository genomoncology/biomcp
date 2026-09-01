---
flow: build
priority: 5
---

# The highest-ancestry frequency line invites an overclaim the same card disclaims

`biomcp get variant rs1426654 population` prints a frequency of 1.0 with no denominator, one line above a figure that was carefully bounded to avoid exactly that. Verified against the repository build `biomcp 0.9.0-dev.6` (`./target/debug/biomcp`) on 2026-09-01:

```
Exome highest ancestry frequency: eas_XX (0.994495)
gnomAD v4 exome grpmax FAF95: 0.98581 (eas)
Genome highest ancestry frequency: 1kg:cdx_XY (1)
gnomAD v4 genome grpmax FAF95: 0.961989 (eas)
gnomAD excludes bottlenecked genetic ancestry groups when selecting grpmax FAF.
```

The genome-side maximum sits on a tiny subgroup. `1kg:cdx_XY` and `hgdp:japanese_XX` reach AF 1.0 on AN between 16 and 56, while the exome side's maximum has AN 39,686. The card prints the raw maximum with no denominator and no floor, and then the next line explains that the neighbouring figure deliberately excludes bottlenecked groups.

A reader who quotes "100% in an ancestry group" from an n=16 subgroup is being invited into a claim the card itself disclaims two lines later.

The data to do better is already present. The JSON carries AN for every row.

Filed from `sdlc/issues/2026-08-27-the-population-summarys-highest-ancestry-line-ignores-denominator-size.md`.

## Required behavior

A frequency presented as the highest observed cannot be read without its denominator.

A maximum drawn from a group too small to support it is either excluded, or shown in a way that makes its size unmissable.

## Done, observably

- The highest-ancestry line for rs1426654 does not present 1.0 as a bare figure.
- A reader can see the sample size behind any frequency the card calls highest.
- The genome and exome sides treat small groups the same way as each other.

## Boundary

This ticket does not change the grpmax FAF95 figure, does not change which populations gnomAD reports, and does not change the JSON's row set. Whether the fix is a floor, a displayed denominator, or a label on small-cohort identifiers is a design decision and is not settled here.
