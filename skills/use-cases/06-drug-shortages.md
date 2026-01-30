# Use Case: Drug Shortage Monitoring

Monitor FDA drug shortages for supply chain planning.

## Scenario

Check current drug shortages to inform clinical decision-making and identify alternatives.

## Workflow

### Step 1: Search Current Shortages

```bash
biomcp openfda shortage search --status current --limit 25
```

List drugs currently in shortage.

### Step 2: Search by Drug Name

```bash
biomcp openfda shortage search --drug "albuterol"
```

Check shortage status for a specific drug.

### Step 3: Filter by Category

```bash
biomcp openfda shortage search --category "Oncology" --status current
```

Focus on shortages in a therapeutic area.

### Step 4: Get Shortage Details

```bash
biomcp openfda shortage get "Carboplatin Injection"
```

Get detailed shortage information including:
- Reason for shortage
- Estimated resolution
- Alternative products
- Manufacturer contact

## Shortage Status Types

| Status | Description |
|--------|-------------|
| Current | Active shortage |
| Resolved | Shortage ended, supply restored |
| To Be Discontinued | Product being withdrawn |

## Common Shortage Reasons

- Demand increase
- Manufacturing issues
- Raw material shortage
- Regulatory action
- Business decision

## Expected Output

Current shortage example:
```
Drug: Carboplatin Injection
Status: Currently in Shortage
Reason: Demand increase for the drug
Manufacturers: Multiple
Estimated Resolution: TBD
```

## Alternative Resources

If BioMCP shortage data is unavailable:
- [FDA Drug Shortages Database](https://www.accessdata.fda.gov/scripts/drugshortages/)
- [ASHP Drug Shortages](https://www.ashp.org/drug-shortages/current-shortages)

## Tips

- Check shortages before prescribing high-risk medications
- Monitor oncology drugs closely (frequent shortages)
- Contact manufacturers directly for supply timelines
- Consider therapeutic alternatives when available
