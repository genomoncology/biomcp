# Semantic Scholar enrichment ignores explicit source selection

Severity: blocking. Article search can report
`semantic_scholar_enabled: false` and still contact Semantic Scholar while
finalizing rows. The behavior was first observed when the supposedly offline
output-footprint corpus made a real Semantic Scholar request.

The planner enables Semantic Scholar only for the default/all and explicit
Semantic Scholar routes, but the shared finalizer enriches candidate rows
unconditionally. PubMed-, PubTator-, and Europe PMC-only routes use that
finalizer. A user therefore cannot rely on explicit source selection to control
which providers receive a query or article identifiers, and the emitted source
status can disagree with actual outbound traffic.

This is separate from enforcing a no-public-network routine gate. Network
isolation makes an accidental request fail; the product must also refrain from
constructing or sending the request when the source is excluded.

Ticket 0926 owns the repair and deletes this issue when it lands.
