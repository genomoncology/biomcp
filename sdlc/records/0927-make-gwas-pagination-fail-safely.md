---
base: 88f8e6ddb937cb5a07852524329a3c14dd01583f
head: ec0003de
---

GWAS search now rejects zero, oversized, overflowing, and out-of-budget
`offset + limit` windows before constructing a provider client. Supported
windows use one extra row only when it can produce a followable offset. The
50-row boundary instead reports a distinct provider-budget truncation state
with no unusable continuation.

GWAS JSON has its own six-field pagination object, while human output gives
different guidance for a followable page, true exhaustion, and the fixed
provider ceiling. Focused tests cover offsets 0, 49, 50, 200, and
`usize::MAX`, exact and overflowing windows, all serialized pagination states,
and construction of the bounded provider request. No routine test uses the
public GWAS service.

The combined 0927/0928 implementation added 129 net `src` lines, below the
two tickets' combined 280-line ceiling.
