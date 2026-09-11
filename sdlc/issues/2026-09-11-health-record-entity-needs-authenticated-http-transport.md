# Health-record entity needs authenticated HTTP transport before it ships there

Found during an outside design review (2026-09-11) of the FHIR client prototype. The ideal state names one patient's own FHIR record as a second destination, reached through the same search and get grammar BioMCP already uses. The review recommended one new `record` entity with clinical kinds and zero new MCP tools.

The review set one gate: record calls may ship over local stdio only, not over the HTTP transport, until the HTTP transport has authenticated, per-user context isolation. Today's docs describe the HTTP transport as unauthenticated and guarded only by hostname. Hostname guarding is not isolation: it does not tell two users' sessions apart, so a record call on that transport cannot be bound to the calling user's own identity.

Outcome needed before the record entity is enabled over HTTP: an authenticated per-user session on the HTTP transport, with every record call bound to an immutable context handle set at session start. A call must not be able to read another user's context handle or drift onto it mid-session.

Also needed: logs for record calls must carry only an allowlist of fields (request id, operation, duration, error category) and nothing else from the record payload, since these calls touch a real patient's own data.

No design decided here. This is a gate for a future ticket, not the ticket itself.
