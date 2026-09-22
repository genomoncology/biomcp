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

BioMCP keeps four kinds of managed local request state: HTTP provider responses
under the resolved cache root's `http/` directory, article-search session
records containing a session token, query terms, PMIDs, and update time under
`sessions/`, recovered directed citation evidence under
`citation-evidence/`, and exact provider documents under `captures/`, such as
the ClinGen CSpec specification document bound to a capture ID. HTTP entries
expire at configured `max_age_secs` (one day by default); article sessions
expire after ten minutes; citation-evidence records expire after thirty days;
provider captures expire after seven days, are capped at 4 MiB each, and retain
at most 64 MiB in total. Opening a store for normal use or inspection physically
removes expired records. `biomcp --json cache stats` reports retained capture
bytes as `provider_capture_bytes`, and `biomcp cache clean` reports the bytes it
frees as `provider_capture_bytes_freed`. `biomcp cache clear --yes` removes the
HTTP, session, and citation-evidence trees; retained provider captures are
bounded by maintenance instead.

BioMCP creates managed local-state directories for the current user only. On
Unix, directories are mode `0700` and regular files are `0600`; on Windows,
the managed root uses a current-user ACL. Reopening a store narrows managed
paths without following links or changing unrelated ancestors, and linked
regular files are rejected before access. These filesystem controls still do
not govern copies retained by an upstream provider.

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
