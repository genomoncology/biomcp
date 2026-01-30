# Use Case: FDA Drug Label Search

Search FDA drug labels for indications, contraindications, and safety information.

## Scenario

Review FDA-approved prescribing information for drugs in a therapeutic area.

## Workflow

### Step 1: Search by Drug Name

```bash
biomcp openfda label search --name "vemurafenib"
```

Find label for a specific drug.

### Step 2: Search by Indication

```bash
biomcp openfda label search --indication "melanoma" --limit 10
```

Find all drugs approved for melanoma.

### Step 3: Filter for Boxed Warnings

```bash
biomcp openfda label search --indication "melanoma" --boxed-warning
```

Find drugs with serious safety warnings.

### Step 4: Compare Drug Labels

```bash
biomcp openfda label search --name "dabrafenib"
biomcp openfda label search --name "trametinib"
```

Compare labels for combination therapy components.

### Step 5: Get Full Label Details

```bash
biomcp openfda label get <label_id>
```

Retrieve complete prescribing information.

## Label Sections

| Section | Content |
|---------|---------|
| Indications | Approved uses |
| Contraindications | When not to use |
| Warnings | Boxed and other warnings |
| Adverse Reactions | Side effect profile |
| Drug Interactions | Concomitant drug issues |
| Dosage | Dosing recommendations |

## Search Parameters

| Parameter | Purpose |
|-----------|---------|
| `--name` | Drug name (brand or generic) |
| `--indication` | Search by approved indication |
| `--boxed-warning` | Filter for boxed warnings |
| `--section` | Search specific label section |

## Expected Output

Melanoma label search results:
- Mekinist (trametinib) - MEK inhibitor
- Keytruda (pembrolizumab) - PD-1 inhibitor
- Dacarbazine - Chemotherapy
- Zelboraf (vemurafenib) - BRAF inhibitor

## Example: Vemurafenib Label

```
Drug: VEMURAFENIB (ZELBORAF)
Application: NDA202429
Manufacturer: Genentech, Inc.
Indication: Unresectable or metastatic melanoma with BRAF V600E mutation
```

## Tips

- Use generic names for comprehensive searches
- Check indications for required biomarker testing
- Review contraindications before prescribing
- Note dosage adjustments for organ impairment
- Cross-reference with adverse event data
