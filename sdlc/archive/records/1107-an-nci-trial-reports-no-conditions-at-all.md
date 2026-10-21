---
base: d25ab2e66a5433a629f71e97f05610fec165ff8b
head: 54dc4e2e77845a5a5b121c315a77a6ae27937838
---

# Report NCI trial conditions

NCI trial search and detail conversion now reads disease names from provider
objects and preserves scalar names containing commas. Unreadable disease entries
reject the response instead of silently producing an incomplete condition list.
