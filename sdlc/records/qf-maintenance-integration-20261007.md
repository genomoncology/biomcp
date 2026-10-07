# Integrate current maintenance checks

Integrate accepted maintenancec3b339c9799999803d5300f5901f633be35cfbf1 into dedicated1.0 baseb2bc5963. Changelog checks and records change; the current OncoKB helper implementation remains unchanged.

The only conflict was tests/test_ci_classify_push_script.py. Keep the upstream seven fresh-baseline path cases and both changelog triggers. Retire the older four-path owner because the broader case table owns those claims. Fresh read-only review accepted the resolved blob6d16c7be80b915a3296ce4a6cf398a095f842848 against both parents and the classifier. No distinct behavior claim was lost.

Fifty affected classifier/changelog checks passed in6.56 seconds on cached macOS Python3.13 before removing the four redundant cases. The remaining46 checks are unchanged. No product check, hosted job, source request or whole maintenance gate ran. Root retains dedicated1.0 integration and push. No maintenance branch or checkout was changed.
