# FAQ - Quick Answers

## Getting Started

**Q: What is BioMCP?**
A: A unified interface to biomedical databases (PubMed, ClinicalTrials.gov, MyVariant, etc.) for researchers and AI assistants.

**Q: Do I need API keys?**
A: No for basic use. Yes for: NCI trials (cancer-specific), AlphaGenome (variant predictions), enhanced cBioPortal features.

**Q: How do I install it?**
A: `uv tool install biomcp` (recommended) or `pip install biomcp-python`

## Common Issues

**Q: "Command not found" after installation**
A: Restart terminal, or use full path: `~/.local/bin/biomcp`

**Q: No results for gene search**
A: Use official symbols (ERBB2 not HER2). Check at [genenames.org](https://www.genenames.org)

**Q: Location search not working**
A: Must provide coordinates: `--latitude 42.3601 --longitude -71.0589`

**Q: Why does the AI use 'think' first?**
A: Required for systematic analysis. Improves search quality and prevents missed connections.

## Search Tips

**Q: How to search variant notations?**
A: Use OR syntax: `--keyword "V600E|p.V600E|c.1799T>A"`

**Q: Include/exclude preprints?**
A: Included by default. Use `--no-preprints` to exclude.

**Q: Search multiple databases?**
A: Use unified search: `search(query="gene:BRAF AND disease:melanoma")`

## Data Questions

**Q: How current is the data?**
A: Daily updates for PubMed/trials, weekly for BioThings, varies for cBioPortal.

**Q: ClinicalTrials.gov vs NCI?**
A: CT.gov = comprehensive, NCI = cancer-focused with biomarker filters (needs API key).

**Q: What's MSI/TMB/VAF?**
A: MSI = Microsatellite Instability, TMB = Tumor Mutational Burden, VAF = Variant Allele Frequency

## Technical

**Q: Rate limits?**
A: ~3 req/sec without keys, higher with keys. NCI = 1000/day with key.

**Q: Cache issues?**
A: Clear with: `rm -rf ~/.biomcp/cache`

**Q: Which Python version?**
A: 3.10+ required

## Quick References

**Common Gene Aliases:**

- HER2 → ERBB2
- PD-L1 → CD274
- c-MET → MET

**City Coordinates:**

- NYC: 40.7128, -74.0060
- Boston: 42.3601, -71.0589
- LA: 34.0522, -118.2437

**Trial Status:**

- RECRUITING = Currently enrolling
- ACTIVE_NOT_RECRUITING = Ongoing
- COMPLETED = Finished

## Getting Help

1. Check this FAQ
2. Read [Troubleshooting](troubleshooting.md)
3. Search [GitHub Issues](https://github.com/genomoncology/biomcp/issues)
4. Ask with version info: `biomcp --version`
