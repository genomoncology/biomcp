---
flow: quickfix
priority: 4
hold: draft for review; do not promote until Ian releases this
---
# Make gene cspec and variant erepo flags do what they say

Three flags on the new ClinGen commands do not behave the way their presence implies, on the development build:

`gene cspec BRCA1` and `--json gene cspec BRCA1` produce byte-identical JSON. The command ignores `--json` because it has no Markdown rendering of the manifest or the criteria list at all, even though `gene cspec BRCA1 --files` does render Markdown. A global modifier that changes nothing is worse than one that is rejected, because the caller believes it took effect.

`variant erepo --detail` produces identical Markdown with and without the flag. The narrative summary and source URL it implies only appear under `--json`.

`variant erepo --input` refuses to run without `--json`, reporting `Error: Invalid argument: variant erepo --input requires --json`, while the single-CAid form has no such restriction. Either the batch form should render Markdown like its sibling, or the asymmetry should be explained where a caller will see it.

Settle each of the three the same way: either the flag does something, or it is rejected with a message that says why. Silent no-ops are the outcome to eliminate.

## Done when

- `--json` on `gene cspec` either changes the output or is rejected with a message naming what is unsupported.
- `variant erepo --detail` either changes the Markdown or is rejected in the Markdown path.
- The `--input` restriction is either lifted or explained in help text and in the error itself.
- No flag on these commands is accepted and then ignored.
