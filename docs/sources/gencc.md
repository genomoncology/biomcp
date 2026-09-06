# GenCC

BioMCP exposes submission-level gene–disease validity assertions from the
[Gene Curation Coalition](https://thegencc.org/) without merging them with the
separate ClinGen section.

Run `biomcp get gene ODC1 gencc` to request the section and `biomcp gencc sync`
to explicitly refresh the local dataset.
`biomcp health --api GenCC` uses a quota-exempt HEAD request and reports that
failures affect the `gene gencc section`; it never downloads or publishes data.

The `gencc` object always contains `assertions`, `total_matching_assertions`,
`truncated`, and `status`. Assertions retain their submitter, disease,
classification, inheritance, dates, criteria/report links, and PubMed
identifiers. At most 100 current assertions are returned; the total and
truncation flag describe the result before that cap. Nullable values serialize
as JSON `null`.

BioMCP downloads GenCC's new-format CSV to a private durable store. A validated
dataset is fresh for seven days. Due refreshes use both ETag and Last-Modified
conditional headers; failed automatic refreshes are retried no more than once per day.
Set `BIOMCP_GENCC_DIR` to an absolute private root when the platform
data directory is unsuitable. Global `--no-cache` does not discard or
force-refresh this dataset.

GenCC publishes the data weekly and asks clients not to poll more than once per
day. The documented limit is 20 successful downloads per IP per day;
conditional 304 responses and HEAD requests do not consume that quota. A stale
positive may be shown with a warning. A stale zero-match result is unavailable
rather than evidence of current absence.

The downloadable data are available under
[CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). Please attribute
GenCC and the contributing submitters. The export excludes restricted OMIM
data. These assertions support research and interpretation; they are not a
diagnosis or a substitute for clinical judgment.
