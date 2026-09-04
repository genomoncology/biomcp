# An ORCID cannot open an exact author corpus

Severity: should-fix

A researcher knowledge base needs to answer one starting question: which works does this researcher claim under this ORCID? BioMCP cannot answer that question today. `biomcp list author` says that Semantic Scholar is the only author source, `get author` rejects `orcid:` identifiers, and a current `get author semanticscholar:<id>` response warns that BioMCP has not established an ORCID link.

A downstream researcher-corpus exercise called the public ORCID API directly, called Semantic Scholar separately, and connected the two through pinned papers and institutional evidence. This work mattered. A name search selected a wrong same-name person in one case. Another researcher had two valid Semantic Scholar profiles. The exercise therefore could not replace identity evidence with a name match.

The cheapest useful addition would treat ORCID as another exact provider. `biomcp get author orcid:<id>` would return the public ORCID record, and `biomcp author papers orcid:<id>` would page through claimed works. BioMCP would keep ORCID and Semantic Scholar identities separate. A caller could use a pinned work and the existing `article authors` command to evaluate possible links.

A later feature could return evidence-backed cross-provider candidates. BioMCP should not merge same-name people automatically. The exact-provider rule already protects callers from that error.

The current negative was verified with `biomcp 0.9.0-dev.6`, `biomcp list author`, and `biomcp get author semanticscholar:<id>` on 2026-09-04. Ticket 1060 deliberately excluded ORCID resolution and named it as follow-up work.
