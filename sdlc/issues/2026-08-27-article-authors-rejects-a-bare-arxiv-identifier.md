# article authors rejects a bare arXiv identifier

`biomcp article authors 2110.01406` errors:

    Error: Invalid argument: Unsupported identifier format for Semantic Scholar
    article helpers: '2110.01406'. Supported: PMID, PMCID, DOI, arXiv, or a
    Semantic Scholar paper ID.

The typed form `article authors arXiv:2110.01406` works. The error message is
instructive, so this is a papercut rather than a trap, but the article family
accepts bare identifiers everywhere else (PMID, PMCID, DOI), and a bare
arXiv number is unambiguous — no other supported form starts with digits and
a dot. Observed 2026-08-27 against 0.9.0-dev.6 while verifying ticket 1060
(capture: experiments/184-biomcp-slide-lab/calls/1060-article-authors.txt).
