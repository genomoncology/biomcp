# Pattern: Gene-in-disease orientation

Use this when the question names a gene and variant class, but the syndrome or disease caused by that gene is still unclear.

```bash
biomcp get gene IPO8
biomcp search article -g IPO8 -k "biallelic loss-of-function Loeys-Dietz Shprintzen-Goldberg thoracic aortic aneurysm" --limit 5
biomcp get gene IPO8 diseases
biomcp article batch 36905820 34010605
```

Interpretation:
- Start with the gene card so aliases, summaries, and `_meta.next_commands` tell you which structured surfaces exist for IPO8.
- Use gene-filtered literature before broad keyword searches; it keeps IPO8 anchored while exploring the loss-of-function and aortopathy clues.
- Check `get gene IPO8 diseases` before committing to a syndrome name, because disease associations can separate known links from candidate or overlapping phenotypes.
- Fetch the focused papers that mention syndromic thoracic aortic aneurysm, Loeys-Dietz, or Shprintzen-Goldberg overlap before making the causal disease statement.
