---
base: 2b73809d
head: faf4312c
---

Ordinary provider clients now use one direct-network policy: public HTTPS by
default, exact explicitly configured fixture or on-prem origins, private-address
DNS rejection, and same-origin redirects. Direct provider clients and shared
cached clients use the same boundary, ignore ambient proxies, and keep
credentials on their approved origin.

Release downloads use a separate reviewed GitHub and GitHub-CDN policy rather
than the ordinary provider client. Tests must declare exact private fixture
origins, which preserves local transport coverage without weakening production
requests. Structural inventories detect new ungoverned client builders and URL
fetch consumers.

Two independent adversarial review rounds accepted the final policy. The
complete release gate passed: 2,947 Rust tests, 630 Python tests, strict docs,
all-feature lint and tests, optimized release build and artifact smoke checks,
and all offline release specifications.
