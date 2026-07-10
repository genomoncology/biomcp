# Live DDInter maintenance

DDInter interaction reads use an installed local bundle, while operators choose
when to replace it. This live verification confirms the explicit maintenance
command can install a real bundle that the normal bounded read then consumes.

## Explicitly synchronize DDInter

Synchronization contacts the real DDInter download service and validates the
complete bundle before making it available to reads.

```bash run id=ddinter-sync exit=0 stream=stderr
../../tools/biomcp-ci ddinter sync
```

```text expect=ddinter-sync contains
Refreshing DDInter data
```

A read after synchronization reports the installed bundle's freshness and its
bounded page size instead of performing another maintenance operation.

```bash
../../tools/biomcp-ci --json drug interactions warfarin | jq -c '{bundle_freshness, returned: .pagination.count, limit: .pagination.limit}' | mustmatch like '{"bundle_freshness":{"status":"fresh"},"limit":25}'
```
