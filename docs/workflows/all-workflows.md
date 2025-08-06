# BioMCP Research Workflows

Quick, practical workflows for common biomedical research tasks.

## 1. Literature Review Workflow

### Quick Start

```bash
# Find key papers on BRAF V600E melanoma therapy
biomcp article search --gene BRAF --disease melanoma \
  --keyword "V600E|therapy|treatment" --limit 50 \
  --format json > braf_papers.json
```

### Full Workflow Script

```python
import asyncio
from biomcp import BioMCPClient

async def literature_review(gene, disease, focus_terms):
    async with BioMCPClient() as client:
        # 1. Get gene context
        gene_info = await client.genes.get(gene)

        # 2. Search by topic
        results = {}
        for term in focus_terms:
            articles = await client.articles.search(
                genes=[gene],
                diseases=[disease],
                keywords=[term],
                limit=30
            )
            results[term] = articles.articles

        # 3. Generate summary
        print(f"\n{gene} in {disease}: Found {sum(len(v) for v in results.values())} articles")
        for topic, articles in results.items():
            print(f"\n{topic}: {len(articles)} articles")
            for a in articles[:3]:
                print(f"  - {a.title[:80]}... ({a.year})")

        return results

# Run it
asyncio.run(literature_review(
    "BRAF",
    "melanoma",
    ["resistance", "combination therapy", "immunotherapy"]
))
```

### Key Points

- Start broad, then narrow by topic
- Use OR syntax for variant notations
- Export results for citation management
- Set up weekly searches for updates

---

## 2. Clinical Trial Matching Workflow

### Quick Start

```bash
# Find trials for EGFR-mutant lung cancer near Boston
biomcp trial search --condition "lung cancer" \
  --term "EGFR mutation" --status RECRUITING \
  --latitude 42.3601 --longitude -71.0589 --distance 100
```

### Patient Matching Script

```python
async def match_patient_to_trials(patient_profile):
    async with BioMCPClient() as client:
        # 1. Search trials with location
        trials = await client.trials.search(
            conditions=[patient_profile['diagnosis']],
            other_terms=patient_profile['mutations'],
            lat=patient_profile['lat'],
            long=patient_profile['long'],
            distance=patient_profile['max_distance'],
            status="RECRUITING"
        )

        # 2. Score trials
        scored = []
        for trial in trials.trials[:20]:
            score = 0

            # Location score
            if trial.distance < 50:
                score += 25

            # Phase score
            if trial.phase == "PHASE3":
                score += 20
            elif trial.phase == "PHASE2":
                score += 15

            # Mutation match
            if any(mut in str(trial.eligibility) for mut in patient_profile['mutations']):
                score += 30

            scored.append((score, trial))

        # 3. Return top matches
        scored.sort(reverse=True, key=lambda x: x[0])
        return [(s, t) for s, t in scored[:5]]

# Example patient
patient = {
    'diagnosis': 'non-small cell lung cancer',
    'mutations': ['EGFR L858R'],
    'lat': 42.3601,
    'long': -71.0589,
    'max_distance': 100
}

matches = asyncio.run(match_patient_to_trials(patient))
```

### Key Points

- Always use coordinates for location search
- Check both ClinicalTrials.gov and NCI sources
- Contact trial sites directly for pre-screening
- Consider travel burden in recommendations

---

## 3. Variant Interpretation Workflow

### Quick Start

```bash
# Get variant annotations
biomcp variant get rs121913529  # By rsID
biomcp variant get "NM_007294.4:c.5266dupC"  # By HGVS

# Search pathogenic variants
biomcp variant search --gene BRCA1 --significance pathogenic
```

### Variant Analysis Script

```python
async def interpret_variant(gene, variant_notation, cancer_type):
    async with BioMCPClient() as client:
        # 1. Get variant details
        try:
            variant = await client.variants.get(variant_notation)
            significance = variant.clinical_significance
            frequency = variant.frequencies.gnomad if hasattr(variant, 'frequencies') else None
        except:
            significance = "Not found"
            frequency = None

        # 2. Search literature
        articles = await client.articles.search(
            genes=[gene],
            variants=[variant_notation],
            diseases=[cancer_type],
            limit=10
        )

        # 3. Find trials
        trials = await client.trials.search(
            conditions=[cancer_type],
            other_terms=[f"{gene} mutation"],
            status="RECRUITING",
            limit=5
        )

        # 4. Generate interpretation
        print(f"\nVariant: {gene} {variant_notation}")
        print(f"Significance: {significance}")
        print(f"Population Frequency: {frequency or 'Unknown'}")
        print(f"Literature: {len(articles.articles)} relevant papers")
        print(f"Clinical Trials: {len(trials.trials)} active trials")

        # Actionability assessment
        if significance in ["Pathogenic", "Likely pathogenic"]:
            if trials.trials:
                print("✓ ACTIONABLE - Clinical trials available")
            else:
                print("⚠ Pathogenic but no targeted trials")

        return {
            'significance': significance,
            'frequency': frequency,
            'articles': len(articles.articles),
            'trials': len(trials.trials)
        }

# Run it
asyncio.run(interpret_variant("BRAF", "p.V600E", "melanoma"))
```

### Key Points

- Check multiple databases (MyVariant, ClinVar via articles)
- Consider cancer type for interpretation
- Look for FDA-approved therapies
- Document tier classification

---

## 4. Quick Integration Patterns

### Batch Processing

```python
# Process multiple queries efficiently
async def batch_analysis(items):
    async with BioMCPClient() as client:
        tasks = []
        for item in items:
            if item['type'] == 'gene':
                tasks.append(client.genes.get(item['id']))
            elif item['type'] == 'variant':
                tasks.append(client.variants.get(item['id']))

        results = await asyncio.gather(*tasks, return_exceptions=True)
        return results
```

### Error Handling

```python
from biomcp.exceptions import NotFoundError, RateLimitError
import time

async def robust_search(search_func, **params):
    retries = 3
    for attempt in range(retries):
        try:
            return await search_func(**params)
        except RateLimitError as e:
            if attempt < retries - 1:
                time.sleep(2 ** attempt)  # Exponential backoff
            else:
                raise
        except NotFoundError:
            return None
```

### Caching Results

```python
from functools import lru_cache
import json

# Simple file-based cache
def cache_results(filename):
    def decorator(func):
        async def wrapper(*args, **kwargs):
            # Check cache
            try:
                with open(filename, 'r') as f:
                    return json.load(f)
            except FileNotFoundError:
                pass

            # Fetch and cache
            result = await func(*args, **kwargs)
            with open(filename, 'w') as f:
                json.dump(result, f)
            return result
        return wrapper
    return decorator

@cache_results('gene_cache.json')
async def get_gene_info(gene):
    async with BioMCPClient() as client:
        return await client.genes.get(gene)
```

---

## Complete Example: Precision Medicine Report

```python
async def generate_precision_medicine_report(patient):
    """Generate comprehensive report for molecular tumor board."""

    async with BioMCPClient() as client:
        report = {
            'patient_id': patient['id'],
            'date': datetime.now().isoformat(),
            'variants': [],
            'trials': [],
            'therapies': []
        }

        # Analyze each variant
        for variant in patient['variants']:
            # Get annotations
            var_info = await robust_search(
                client.variants.search,
                gene=variant['gene'],
                hgvs=variant['hgvs']
            )

            # Search literature
            articles = await client.articles.search(
                genes=[variant['gene']],
                diseases=[patient['cancer_type']],
                keywords=['therapy', 'treatment'],
                limit=5
            )

            # Find trials
            trials = await client.trials.search(
                conditions=[patient['cancer_type']],
                other_terms=[f"{variant['gene']} mutation"],
                status="RECRUITING",
                limit=3
            )

            report['variants'].append({
                'variant': variant,
                'annotation': var_info,
                'relevant_articles': len(articles.articles),
                'available_trials': len(trials.trials)
            })

            report['trials'].extend(trials.trials)

        # Generate summary
        print(f"\nPrecision Medicine Report - {patient['id']}")
        print(f"Cancer Type: {patient['cancer_type']}")
        print(f"Variants Analyzed: {len(report['variants'])}")
        print(f"Clinical Trials Found: {len(report['trials'])}")

        # Prioritize actionable findings
        actionable = [v for v in report['variants']
                     if v['available_trials'] > 0]

        if actionable:
            print(f"\n✓ {len(actionable)} ACTIONABLE variants with trial options")

        return report

# Example usage
patient = {
    'id': 'PT001',
    'cancer_type': 'lung adenocarcinoma',
    'variants': [
        {'gene': 'EGFR', 'hgvs': 'p.L858R'},
        {'gene': 'TP53', 'hgvs': 'p.R273H'}
    ]
}

report = asyncio.run(generate_precision_medicine_report(patient))
```

---

## Tips for All Workflows

1. **Always start with the think tool** (for AI assistants)
2. **Use official gene symbols** - check genenames.org
3. **Batch API calls** when possible
4. **Handle errors gracefully** - APIs can be unavailable
5. **Cache frequently accessed data** - respect rate limits
6. **Document your process** - for reproducibility

## Next Steps

- [Command Reference](../reference/quick-reference.md)
- [API Documentation](../apis/python-sdk.md)
- [Troubleshooting](../troubleshooting.md)
