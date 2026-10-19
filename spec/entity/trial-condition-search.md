# Trial Condition Search

## Condition-First Search

Condition search should still look like a trial table, not a blob of text, and
the visible query echo should confirm which narrowing path ran.

```bash
../../tools/biomcp-ci search trial -c melanoma -s recruiting --limit 3 | mustmatch like '# Trial Search Results
Query: condition=melanoma, status=recruiting
|NCT ID|Title|Status|Phase|Conditions|'
```
