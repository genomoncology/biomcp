# Visible metadata recovery replies

Captured on 2026-10-19. The PubTator reply is an HTTP 400 response for PMID 42891779; the separately captured 42894377 response has the same body. The Europe PMC replies are the exact search_by_pmid requests, with no resultType parameter. Adjacent JSON receipts retain request URLs, status and body hashes. Bodies remain unmodified. These replies demonstrate a coverage gap for these requests; the PubTator request shape also matches existing successful source captures.

The test constructs its PubMed search rows, empty search replies from other providers, and successful Europe PMC metadata explicitly as synthetic scaffolding. Those replies are not upstream recordings. A two-party response barrier observes overlapping recovery requests without a performance threshold.
