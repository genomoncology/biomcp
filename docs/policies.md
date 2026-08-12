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

BioMCP keeps two kinds of managed local request state: HTTP provider responses
under the resolved cache root's `http/` directory, and article-search session
records containing a session token, query terms, PMIDs, and update time under
`sessions/`. HTTP entries expire at configured `max_age_secs` (one day by
default); article sessions expire after ten minutes. Opening either store for
normal use or inspection physically removes expired records. `biomcp cache
clean` performs targeted maintenance and `biomcp cache clear --yes` removes
both managed trees.

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
