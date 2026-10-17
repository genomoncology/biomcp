---
flow: build
priority: 2
deps: [2146]
---

# 2150: Find a usable copy of every supplement

Status: OPEN.

Milestone: 0.9.3

## Outcome

When Europe PMC returns an empty supplement and no other copy is listed, BioMCP looks for the file in the PMC open-data bucket and on the PMC article page, lists it when found, and says in the assets manifest when no non-empty copy exists. 2146 covers a spreadsheet; this ticket proves the fix on a PDF and a Word file too. A copy whose bytes do not match its file name's type counts as unusable, the same as an empty copy, and gets an explicit unusable-asset outcome in `assets` and `asset` instead of exit 0.

## Evidence

Reported 2026-10-16 by the research team on the released 0.9.2, after 2146 was filed.

- PDF: PMID 27149063, supplement `pone.0153788.s001.pdf`. The Europe PMC asset exits 0 and writes zero bytes. The PMC OA alternative already listed writes 18,070 bytes, SHA-256 `3e3ab5a0033e5cf20355197da561ffecdc8c435c0c9ad69cca3cf1e685d89ef0`.
- Word: PMID 30224644, supplement `NIHMS1501645-supplement-2.docx`. The Europe PMC asset exits 0 and writes zero bytes, and its manifest advertises the empty checksum. The manifest lists no other copy, so the supplement cannot be read at all.
- Wrong type: PMID 33332384, supplement `pgen.1009231.s004.docx`, exits 0 and writes 40,919 bytes of HTML (a vendor page), SHA-256 `1ea833ea0c5ce27d34fb9af69cc39fda31c4e058fd60043c56ba446fd96687ee`. The PMC OA manifest serves the same bytes, so the upstream file is wrong, and BioMCP passes it on with no warning.

## Work

- Reproduce both cases on main after 2146.
- For a supplement whose only listed copy is Europe PMC, check the PMC open-data bucket and the PMC article page for the same file name, and add any copy found.
- Check each downloaded file's leading bytes against its extension (docx, xlsx and zip archives, PDF, and so on). Treat a mismatch, such as HTML under a .docx name, as unusable with its own reason: exit nonzero from `asset`, write no file, and try the next copy.
- For the wrong-type case, also check Europe PMC and the PMC article page for a good copy.
- When every copy is empty, wrong-type or missing, mark the asset in the manifest with that reason.

## Acceptance

1. Recorded-reply tests cover the PDF case (the PMC OA copy is chosen) the Word case (another official copy is found, or the manifest says none exists), and the wrong-type case (the HTML file is refused with the unusable reason).
2. The changelog carries a bullet for this ticket.
