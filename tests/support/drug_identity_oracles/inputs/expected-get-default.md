# sampledrug

DrugBank ID: DB-SYN1
Safety (OpenFDA FAERS): unavailable

More:
  biomcp get drug sampledrug approvals   - Drugs@FDA approval history
  biomcp get drug sampledrug label   - approved-indication and FDA label detail beyond the base card
  biomcp get drug sampledrug regulatory --region us   - approval and supplement history; use only if the base card lacks approval context

All:
  biomcp get drug sampledrug all --region us

See also:
  biomcp search article --drug sampledrug --type review --limit 5   - supplement sparse structured data with review literature for indication context
  biomcp drug trials sampledrug
  biomcp drug adverse-events sampledrug   - inspect safety reports and adverse-event signal
  biomcp search pgx -d sampledrug   - pharmacogenomics interactions

[DrugBank](https://go.drugbank.com/drugs/DB-SYN1)
