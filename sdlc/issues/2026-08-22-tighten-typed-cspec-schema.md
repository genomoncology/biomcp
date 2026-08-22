# Typed CSpec schema leaves mode constraints to runtime

`gene_cspec` advertises unbounded `gene`, and its `version_iri`, `capture_id`,
and `files` mode constraints are not represented in its typed MCP schema.
This was found while surveying typed tools for ticket 1031 and is outside that
ERepo schema change.
