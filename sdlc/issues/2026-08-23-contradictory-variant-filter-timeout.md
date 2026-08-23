# Intermittent contradictory variant-filter timeout

The `json_error_contract::contradictory_variant_filters_fail_before_myvariant_contact`
test has reported an intermittent timeout in the variant-search validation path.
It lives in the JSON error contract tests and is outside the ERepo guidance change.
The focused test passed during this review, so this issue records the flaky failure
for follow-up without changing its timeout or assertions here.

2026-08-23: left as a watch item per Ian's triage ruling — file it only if it recurs.
