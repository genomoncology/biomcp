---
flow: build
priority: 3
hold: draft for review; do not promote until Ian releases this
---
# Accept a short version for gene cspec

Selecting a criteria specification version requires pasting a full resource IRI copied out of the manifest, roughly ninety characters:

```
gene cspec BRCA1 --version 'https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN092/version/1.2.1'
```

The manifest already lists the available versions, and the version number at the end of each IRI is what a person actually means. Accepting `--version 1.2.1` would remove a copy-paste step from a command a curation team runs repeatedly, and would make the command reasonable to type from memory or to generate in a script.

The full IRI must keep working, because it is unambiguous and it is what the manifest returns.

## Done when

- `--version 1.2.1` selects the matching specification for the named gene.
- The full resource IRI continues to work exactly as it does today.
- A short version that matches nothing, or that is ambiguous for the gene, fails with a message listing the versions that are available.
