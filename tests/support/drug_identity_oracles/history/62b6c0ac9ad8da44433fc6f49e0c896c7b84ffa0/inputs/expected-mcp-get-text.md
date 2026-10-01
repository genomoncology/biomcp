# sampledrug - approvals

## Drugs@FDA Approvals

No approvals found in Drugs@FDA for this query.

More:
  biomcp get drug sampledrug label   - approved-indication and FDA label detail beyond the base card
  biomcp get drug sampledrug regulatory --region us   - approval and supplement history; use only if the base card lacks approval context
  biomcp get drug sampledrug safety --region us   - regulatory safety detail; use `biomcp drug adverse-events <name>` first when you want post-marketing signal

All:
  biomcp get drug sampledrug all --region us

See also:
  biomcp search article --drug sampledrug --type review --limit 5   - supplement sparse structured data with review literature for indication context
  biomcp drug trials sampledrug
  biomcp drug adverse-events sampledrug   - inspect safety reports and adverse-event signal
  biomcp search pgx -d sampledrug   - pharmacogenomics interactions

[DrugBank](https://go.drugbank.com/drugs/DB-SYN1)


## Sources
- Overview: MyChem.info
- Regulatory: OpenFDA Drugs@FDA
- Drugs@FDA Approvals: OpenFDA Drugs@FDA

## Next commands
- `biomcp get drug sampledrug label`
- `biomcp get drug sampledrug regulatory --region us`
- `biomcp get drug sampledrug safety --region us`
- `biomcp get drug sampledrug all --region us`
- `biomcp search article --drug sampledrug --type review --limit 5`
- `biomcp drug trials sampledrug`
- `biomcp drug adverse-events sampledrug`
- `biomcp search pgx -d sampledrug`