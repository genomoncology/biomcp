# Privacy Policy

BioMCP is a read-only biomedical CLI and MCP server. This page describes
BioMCP's own privacy posture for the Anthropic directory and links to the
provider-specific terms that still apply when a query touches an upstream API.

## BioMCP data handling

- BioMCP does not add telemetry, analytics, or remote log upload.
- BioMCP does not operate a hosted control plane that collects or stores your
  prompts, queries, API keys, or results.
- BioMCP sends tool-request data only to the upstream biomedical providers
  needed to satisfy the command you run.
- BioMCP is read-only. It does not modify external systems, write back to
  source databases, or create side effects in third-party services.

## Anthropic and upstream providers

- When you use BioMCP through Claude or another MCP client, Anthropic or that
  client platform may process tool inputs and outputs under its own policies.
  BioMCP does not control that layer.
- Upstream providers may log requests, apply their own retention windows, and
  enforce their own privacy policies and terms of service.
- Use the [Source licensing reference](reference/source-licensing.md) for
  provider-specific links, auth expectations, and reuse notes.

## API keys and credentials

- API keys are supplied by the user at runtime and are not committed to this
  repository or bundled into the `.mcpb` package.
- BioMCP passes configured keys through to upstream APIs only when the relevant
  command requires them.
- Keep sensitive keys scoped to the least privilege and rotation policy your
  organization allows.

## Retention and sensitive data

BioMCP stores reusable HTTP responses, recovered full text, directed citation evidence and exact provider captures in `<cache_root>/biomcp.db`. Bodies larger than 1 MiB use content-addressed files under `blobs/`. Article-search sessions remain under `sessions/`; saved readable full text remains under `downloads/`.

HTTP entries expire at configured `max_age_secs` (one day by default); article sessions expire after ten minutes; citation evidence expires after thirty days. Provider captures expire after seven days, are capped at 4 MiB each, and retain at most 64 MiB in total. Capture publication and explicit cleanup enforce those capture limits independently of the reusable-cache budget. `biomcp --json cache stats` reports retained capture bytes as `provider_capture_bytes`; `cache clean` reports expired capture bytes as `provider_capture_bytes_freed`. Ordinary `cache clear --yes` removes reusable entries and sessions while preserving provider captures, saved downloads and explicit exports.

BioMCP creates managed directories for the current user only. Unix directories use mode `0700` and files use `0600`; Windows uses a current-user ACL. Opening repairs the managed root without changing unrelated ancestors. Existing Unix SQLite files with broader permissions are refused rather than reopened for repair. Symlinks and hardlinked database or blob files are rejected. Optional-cache failures fall through to provider retrieval with a local-cache diagnostic. These controls do not govern upstream copies.

The new store does not import or delete legacy cache stores. Older binaries continue using their own stores until upgraded. `cache stats` names legacy sizes; after upgrading and stopping older processes, `cache clean-legacy --yes` explicitly removes retired stores while preserving downloads, the database, blobs and sessions.

For an invocation that should leave no managed request state, use
`--no-cache`. It bypasses both HTTP cache reads/writes and article sessions;
combining it with `--session` is rejected. Explicit downloads, local study
datasets, and requested captures remain durable output, and providers can still
log or retain the requests they receive.

These local controls do not govern provider systems. Anthropic/Claude and upstream providers may retain
request data according to their own privacy policies. Do not send protected
health information (PHI) or other sensitive patient data to third-party APIs
unless your organization has approved that workflow.

## Output and clinical use

BioMCP is a research and workflow aid. Validate clinically relevant information
against primary sources, official labels, and your institution's policies
before using it for patient care or operational decisions.

## Contact and support

For privacy questions or support, use
[GitHub issues](https://github.com/genomoncology/biomcp/issues) or the
[troubleshooting guide](troubleshooting.md).
