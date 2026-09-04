# One exact-variant article request can exceed two minutes without returning a result

The following command exceeded a 130-second process limit on `biomcp 0.9.0-dev.6` on 2026-09-04. It produced no JSON before the process ended with exit 124:

```bash
timeout 130 biomcp --json variant articles "ODC1 c.1342A>T" --limit 10
```

Adding `--verify-identity` produced the same outcome. The isolated annotation route completed in 21.26 seconds:

```bash
biomcp --json variant articles "ODC1 c.1342A>T" --strategy annotation --limit 10
```

The default union expands several exact aliases across PubMed, Europe PMC, Semantic Scholar, and PubTator. `src/entities/article/variant_search.rs::strict_provider_candidates` awaits those request plans one after another. The command then runs annotation, lexical, citation, and visible-row enrichment phases. BioMCP caps logical calls, but it does not cap the total wall-clock duration. Several ordinary provider timeouts can therefore accumulate into a command that exceeds an agent runner's larger timeout.

## Recommended design

Give the complete helper a 60-second wall-clock budget. Run independent provider work concurrently within each provider's rate limit. Stop launching work when the remaining budget cannot cover it. Return every completed row plus source statuses that name unfinished routes. Preserve a nonzero exit only when no route produced a usable result.

The cost is that a slow provider can produce an explicitly partial result more often. The current behavior produces no usable result at all when the caller's process limit wins.

## Done, observably

- The ODC1 command returns or exits within the documented total budget.
- A slow fixture proves that completed routes survive when another route reaches the total budget.
- JSON marks the response incomplete and names each unfinished route.
- `--strategy annotation` and `--strategy lexical` keep their diagnostic value.
- The existing logical work limits and provider rate limits remain enforced.
