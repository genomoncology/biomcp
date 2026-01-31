# Pattern: Drug Shortage Monitoring

Monitor FDA drug shortages for supply chain planning.

## Workflow

1. **Check specific drug** - `biomcp openfda shortage search --drug <name>`
2. **List current shortages** - `biomcp openfda shortage search --status current`
3. **Get shortage details** - Review reason, expected resolution, alternatives

## Shortage Status

| Status       | Meaning                 |
| ------------ | ----------------------- |
| Current      | Active shortage         |
| Resolved     | Supply restored         |
| Discontinued | Product being withdrawn |

## Common Reasons

- Demand increase
- Manufacturing issues
- Raw material shortage
- Regulatory action

## Tips

- Check shortages before prescribing high-risk medications
- Monitor oncology drugs closely (frequent shortages)
- Look for therapeutic alternatives when shortage is prolonged
